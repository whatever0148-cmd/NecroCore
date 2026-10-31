//! necro-kiro-bridge: `NecroAutopsyReport` -> Kiro spec text and patch proposals.
//!
//! The live path spawns `kiro-cli chat --no-interactive --trust-tools= <prompt>`
//! (the Kiro CLI, 2.34 at the time of writing) with a hard timeout and no tool
//! access, so the model can only write text. The offline path is deterministic
//! and derived entirely from the report, so tests and the benchmark never block
//! on a network or a login.
use necro_core::{NecroAutopsyReport, PathologyKind};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::AsyncReadExt;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PatchProposal {
    pub pathology_index: usize,
    pub title: String,
    pub rationale: String,
    pub proposed_change: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpecProposal {
    /// "kiro-cli" or "offline".
    pub source: String,
    pub requirements_md: String,
    pub patches: Vec<PatchProposal>,
    pub elapsed_ms: u128,
}

#[derive(Clone, Debug)]
pub struct KiroBridge {
    pub cli: PathBuf,
    pub timeout: Duration,
    pub live: bool,
}

impl Default for KiroBridge {
    fn default() -> Self {
        let home = std::env::var("HOME").unwrap_or_default();
        KiroBridge { cli: PathBuf::from(format!("{home}/.local/bin/kiro-cli")), timeout: Duration::from_secs(90), live: false }
    }
}

impl KiroBridge {
    pub fn offline() -> Self {
        KiroBridge { live: false, ..Default::default() }
    }
    pub fn live(timeout: Duration) -> Self {
        KiroBridge { live: true, timeout, ..Default::default() }
    }
    pub fn cli_available(&self) -> bool {
        self.cli.is_file()
    }

    /// Builds the prompt sent to Kiro. Kept public so the spec can quote it.
    pub fn prompt_for(report: &NecroAutopsyReport) -> String {
        let mut p = String::new();
        p.push_str("You are helping modernize a legacy program. Below is a static autopsy report in JSON. ");
        p.push_str("Reply with Markdown only, no tool use. Section 1: EARS requirements (WHEN ... THE SYSTEM SHALL ...) for each pathology. ");
        p.push_str("Section 2: one concrete patch proposal per pathology, naming the block and the line. Keep it under 60 lines.\n\n");
        p.push_str(&serde_json::to_string_pretty(report).unwrap_or_default());
        p
    }

    /// Deterministic proposal derived from the report alone.
    pub fn offline_proposal(report: &NecroAutopsyReport) -> SpecProposal {
        let patches = report
            .pathologies
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let (title, change) = match p.kind {
                    PathologyKind::Ghost => (
                        format!("Exorcise unreachable block `{}`", p.block),
                        format!("Detach `{}` (line {}, bytes {}..{}) from the lifted module; keep the source text in the autopsy for the record.", p.block, p.span.line, p.span.byte_start, p.span.byte_end),
                    ),
                    PathologyKind::Vampire => (
                        format!("Bound `{}` in loop {}", p.variable.clone().unwrap_or_default(), p.blocks.join(", ")),
                        format!("Insert a guard before `{}` that leaves the loop once `{}` reaches {} and record the bound in the report.", p.block, p.variable.clone().unwrap_or_default(), necro_core::heal::HEAL_CAP),
                    ),
                    PathologyKind::Zombie => (
                        format!("Give loop {} a fuel counter", p.blocks.join(", ")),
                        format!("Route every back edge into `{}` through a guard that decrements a fuel counter from {} and halts at zero.", p.block, necro_core::heal::HEAL_FUEL),
                    ),
                };
                PatchProposal { pathology_index: i, title, rationale: p.detail.clone(), proposed_change: change }
            })
            .collect();
        SpecProposal { source: "offline".into(), requirements_md: report.requirements_markdown.clone(), patches, elapsed_ms: 0 }
    }

    /// Produces a proposal. The live path falls back to the offline proposal on
    /// any failure (missing binary, non-zero exit, timeout, empty output) and says
    /// so in `source`.
    pub async fn specify(&self, report: &NecroAutopsyReport) -> SpecProposal {
        if !self.live {
            return Self::offline_proposal(report);
        }
        let start = std::time::Instant::now();
        match self.run_live(report).await {
            Ok(text) if !text.trim().is_empty() => {
                let mut p = Self::offline_proposal(report);
                p.source = "kiro-cli".into();
                p.requirements_md = text;
                p.elapsed_ms = start.elapsed().as_millis();
                p
            }
            Ok(_) => {
                let mut p = Self::offline_proposal(report);
                p.source = "offline (kiro-cli returned no text)".into();
                p.elapsed_ms = start.elapsed().as_millis();
                p
            }
            Err(e) => {
                let mut p = Self::offline_proposal(report);
                p.source = format!("offline ({e})");
                p.elapsed_ms = start.elapsed().as_millis();
                p
            }
        }
    }

    async fn run_live(&self, report: &NecroAutopsyReport) -> Result<String, String> {
        if !self.cli_available() {
            return Err(format!("kiro-cli not found at {}", self.cli.display()));
        }
        let prompt = Self::prompt_for(report);
        let mut child = tokio::process::Command::new(&self.cli)
            .args(["chat", "--no-interactive", "--trust-tools=", "--wrap", "never"])
            .arg(&prompt)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| format!("spawn failed: {e}"))?;
        let mut stdout = child.stdout.take().ok_or("no stdout")?;
        let read = async {
            let mut buf = String::new();
            stdout.read_to_string(&mut buf).await.map_err(|e| e.to_string())?;
            let status = child.wait().await.map_err(|e| e.to_string())?;
            if !status.success() {
                return Err(format!("kiro-cli exited with {status}"));
            }
            Ok(buf)
        };
        match tokio::time::timeout(self.timeout, read).await {
            Ok(r) => r,
            Err(_) => Err(format!("timed out after {:?}", self.timeout)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use necro_core::{Pathology, Span};

    fn report() -> NecroAutopsyReport {
        let p = Pathology {
            kind: PathologyKind::Zombie,
            label: "Zombie".into(),
            block: "MAIN-PARA.UNTIL-20".into(),
            span: Span::new(10, 40, 20),
            variable: Some("WS-EOF".into()),
            blocks: vec!["MAIN-PARA.UNTIL-20".into()],
            cfg_edges: vec![],
            detail: "exit reads only WS-EOF".into(),
            suggested_fix: "set it".into(),
        };
        NecroAutopsyReport {
            program: "T".into(),
            origin: "cobol".into(),
            block_count: 1,
            edge_count: 0,
            cfg_edges: vec![],
            pathologies: vec![p],
            lifter_warnings: vec![],
            requirements_markdown: "# R".into(),
        }
    }

    #[tokio::test]
    async fn offline_is_deterministic() {
        let b = KiroBridge::offline();
        let a = b.specify(&report()).await;
        let c = b.specify(&report()).await;
        assert_eq!(a, c);
        assert_eq!(a.source, "offline");
        assert_eq!(a.patches.len(), 1);
    }

    #[tokio::test]
    async fn live_with_missing_binary_falls_back_fast() {
        let b = KiroBridge { cli: PathBuf::from("/nonexistent/kiro-cli"), timeout: Duration::from_millis(500), live: true };
        let t = std::time::Instant::now();
        let p = b.specify(&report()).await;
        assert!(p.source.starts_with("offline"));
        assert!(t.elapsed() < Duration::from_secs(2));
    }

    #[tokio::test]
    async fn live_timeout_is_enforced() {
        // `sleep` stands in for a hung CLI: it accepts the args and never answers.
        let b = KiroBridge { cli: PathBuf::from("/bin/sleep"), timeout: Duration::from_millis(300), live: true };
        let t = std::time::Instant::now();
        let p = b.specify(&report()).await;
        assert!(p.source.contains("offline"), "{}", p.source);
        assert!(t.elapsed() < Duration::from_secs(3));
    }
}
