#!/usr/bin/env python3
"""Shell side of the Kiro hooks in .kiro/hooks and .kiro/agents.

Kiro pipes the hook event as JSON on stdin. Each subcommand reads it, appends one
line to benchmarks/hook-firings.log, and does one job. Exit 0 adds stdout to the
agent's context. Any other exit sends stderr to the agent, and for PreToolUse it
blocks the tool call.

  guard-benchmarks  PreToolUse: refuse hand edits to saved benchmark numbers
  voice             PostToolUse: judges' text has no em dashes, semicolons or buzzwords
  sync-page         PostToolUse: rebuild the hosted page when its sources change
  crate-tests       PostToolUse: run the tests of the crate that was just edited
  session-brief     SessionStart / AgentSpawn: git state, last proof, open spec tasks
  task-proof        PostTaskExecution: full test suite plus a 1,000-vector proof

Run by hand with --file PATH to check a file without Kiro, for example
  python3 scripts/hooks/hook.py voice --file README.md
"""
import json
import os
import re
import subprocess
import sys
from datetime import datetime, timezone

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
LOG = os.path.join(ROOT, "benchmarks", "hook-firings.log")
BENCH = "benchmarks/2026-11-15-live-kiro.json"


def read_event(argv):
    if "--file" in argv:
        path = argv[argv.index("--file") + 1]
        return {"hook_event_name": "manual", "tool_input": {"path": path}, "by_hand": True}
    raw = sys.stdin.read() if not sys.stdin.isatty() else ""
    try:
        return json.loads(raw) if raw.strip() else {}
    except json.JSONDecodeError:
        return {"unparsed": raw[:200]}


def rel_path(ev):
    ti = ev.get("tool_input") or {}
    p = ti.get("path") or ti.get("file_path") or ev.get("file_path") or ""
    if p.startswith(ROOT):
        p = p[len(ROOT):].lstrip("/")
    return p


def log(name, ev, outcome):
    entry = {
        "hook": name,
        "event": ev.get("hook_event_name", "?"),
        "tool": ev.get("tool_name"),
        "path": rel_path(ev) or None,
        "session": ev.get("session_id"),
        "by_hand": ev.get("by_hand", False),
        "outcome": outcome,
    }
    stamp = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    with open(LOG, "a") as f:
        f.write(f"{stamp} {name} {json.dumps({k: v for k, v in entry.items() if v is not None})}\n")


def run(cmd, timeout=600):
    p = subprocess.run(cmd, cwd=ROOT, shell=True, capture_output=True, text=True, timeout=timeout)
    return p.returncode, (p.stdout + p.stderr)


# ---------------------------------------------------------------- guard-benchmarks
def guard_benchmarks(ev):
    p = rel_path(ev)
    if re.fullmatch(r"benchmarks/.+\.(json|txt)", p) and not p.endswith("hook-firings.log"):
        log("guard-benchmarks", ev, "blocked")
        print(
            f"Blocked: {p} is a saved run. NecroCore never types numbers by hand. "
            "Regenerate it with `cargo run --release --bin necro-cli -- benchmark "
            "--vectors 10000 --json <file>` and cite the new file instead.",
            file=sys.stderr,
        )
        return 2
    log("guard-benchmarks", ev, "allowed")
    return 0


# ---------------------------------------------------------------- voice
VOICE_FILES = re.compile(r"(deliverables/.+\.md|README\.md)$")
BUZZ = [
    "leverage", "leveraging", "seamless", "seamlessly", "robust", "cutting-edge", "cutting edge",
    "game-changer", "game changer", "revolutionize", "revolutionary", "unlock", "unleash", "empower",
    "delve", "synergy", "state-of-the-art", "next-generation", "groundbreaking", "supercharge",
    "elevate", "harness the power", "in today's fast-paced", "best-in-class", "world-class",
]


def voice_problems(text):
    problems = []
    in_fence = False
    for n, line in enumerate(text.splitlines(), 1):
        if line.lstrip().startswith("```"):
            in_fence = not in_fence
            continue
        if in_fence or line.lstrip().startswith("|---"):
            continue
        prose = re.sub(r"`[^`]*`", "", line)              # inline code is not prose
        prose = re.sub(r"\]\([^)]*\)", "]", prose)         # link targets are not prose
        prose = re.sub(r"https?://\S+", "", prose)
        if "—" in prose:
            problems.append(f"{n}: em dash")
        if re.search(r"\s–\s", prose):
            problems.append(f"{n}: en dash used as a dash")
        if ";" in prose and not prose.lstrip().startswith(("<!--", "&")):
            problems.append(f"{n}: semicolon")
        low = prose.lower()
        for b in BUZZ:
            if re.search(r"(?<![a-z])" + re.escape(b) + r"(?![a-z])", low):
                problems.append(f"{n}: buzzword '{b}'")
    return problems


def voice(ev):
    p = rel_path(ev)
    if not VOICE_FILES.search(p or ""):
        log("voice", ev, "skipped")
        return 0
    full = os.path.join(ROOT, p)
    if not os.path.exists(full):
        log("voice", ev, "missing")
        return 0
    probs = voice_problems(open(full, encoding="utf-8").read())
    if probs:
        log("voice", ev, f"{len(probs)} problems")
        print(f"{p} breaks the judges' voice rules (.kiro/steering/voice.md). Fix these lines:", file=sys.stderr)
        for pr in probs[:40]:
            print("  " + pr, file=sys.stderr)
        return 1
    log("voice", ev, "clean")
    print(f"{p}: voice check clean")
    return 0


# ---------------------------------------------------------------- sync-page
PAGE_SOURCES = re.compile(r"(crates/necro-daemon/src/ui\.html|crates/necro-daemon/src/lib\.rs|necro-cli/src/observatory\.rs)$")


def sync_page(ev):
    p = rel_path(ev)
    if not PAGE_SOURCES.search(p or ""):
        log("sync-page", ev, "skipped")
        return 0
    out = []
    for target in ("docs/index.html", "observatory/index.html"):
        code, text = run(f"cargo run -q --release --bin necro-cli -- observatory --out {target} --bench {BENCH}")
        if code != 0:
            log("sync-page", ev, "build failed")
            print(f"Rebuilding {target} failed after {p} changed:\n{text[-1500:]}", file=sys.stderr)
            return 1
        out.append(text.strip())
    log("sync-page", ev, "rebuilt")
    print("Hosted page rebuilt: " + " | ".join(out))
    return 0


# ---------------------------------------------------------------- crate-tests
def crate_tests(ev):
    p = rel_path(ev)
    m = re.match(r"crates/(necro-[a-z-]+)/", p or "")
    if not m or not p.endswith(".rs"):
        log("crate-tests", ev, "skipped")
        return 0
    crate = m.group(1)
    code, text = run(f"cargo test -q --release -p {crate}")
    tail = "\n".join(text.strip().splitlines()[-15:])
    if code != 0:
        log("crate-tests", ev, f"{crate} failed")
        print(f"cargo test -p {crate} failed after editing {p}:\n{tail}", file=sys.stderr)
        return 1
    passed = sum(int(x) for x in re.findall(r"(\d+) passed", text))
    log("crate-tests", ev, f"{crate} {passed} passed")
    print(f"{crate}: {passed} tests passed")
    return 0


# ---------------------------------------------------------------- session-brief
def session_brief(ev):
    _, status = run("git status --short | head -15")
    _, head = run("git log --oneline -3")
    open_tasks = []
    specs = os.path.join(ROOT, ".kiro", "specs")
    for d in sorted(os.listdir(specs)):
        t = os.path.join(specs, d, "tasks.md")
        if os.path.exists(t):
            n = sum(1 for line in open(t) if re.match(r"\s*- \[ \]", line))
            if n:
                open_tasks.append(f"{d}: {n} open")
    try:
        rows = json.load(open(os.path.join(ROOT, BENCH)))
        v = sum(r["parity"]["vectors"] for r in rows)
        m = sum(r["parity"]["matched"] for r in rows)
        found = sum(r["ghosts"] + r["vampires"] + r["zombies"] for r in rows)
        left = sum(r["healed_remaining"] for r in rows)
        bridges = sorted({r.get("bridge_source", "?") for r in rows})
        proof = f"{m} of {v} vectors matched, {found} pathologies found, {left} left after heal, bridge {'/'.join(bridges)}"
    except Exception as e:  # noqa: BLE001
        proof = f"(could not read {BENCH}: {e})"
    log("session-brief", ev, "ok")
    print("NecroCore session brief")
    print("Last commits:\n" + head.strip())
    print("Uncommitted:\n" + (status.strip() or "(clean)"))
    print("Open spec tasks: " + (", ".join(open_tasks) or "none"))
    print(f"Last saved proof ({BENCH}): {proof}")
    print("Rules: numbers come only from necro-cli runs. Judges' text follows .kiro/steering/voice.md.")
    return 0


# ---------------------------------------------------------------- task-proof
def task_proof(ev):
    code, text = run("cargo test -q --release --workspace", timeout=900)
    if code != 0:
        log("task-proof", ev, "tests failed")
        print("A spec task finished but the workspace tests fail:\n" + text[-2000:], file=sys.stderr)
        return 1
    code, text = run("cargo run -q --release --bin necro-cli -- benchmark --vectors 1000 --rom-steps 200", timeout=900)
    tail = "\n".join(text.strip().splitlines()[-12:])
    if code != 0 or "overall parity: 100.000%" not in text:
        log("task-proof", ev, "proof failed")
        print("A spec task finished but the 1,000-vector proof did not pass:\n" + tail, file=sys.stderr)
        return 1
    log("task-proof", ev, "ok")
    print("Spec task closed with tests green and the proof passing:\n" + tail)
    return 0


JOBS = {
    "guard-benchmarks": guard_benchmarks,
    "voice": voice,
    "sync-page": sync_page,
    "crate-tests": crate_tests,
    "session-brief": session_brief,
    "task-proof": task_proof,
}

if __name__ == "__main__":
    if len(sys.argv) < 2 or sys.argv[1] not in JOBS:
        print(__doc__, file=sys.stderr)
        sys.exit(64)
    sys.exit(JOBS[sys.argv[1]](read_event(sys.argv[2:])))
