//! PhantomArcade. All the behaviour comes from the shared crates. This file is only what makes this app itself:
//! its name, the views it shows and its default port.
use clap::Parser;
use necro_daemon::AppSpec;
use std::path::PathBuf;

const SPEC: AppSpec = AppSpec { id: "phantomarcade", title: "PhantomArcade", tabs: &["arcade", "proof"], port: 4302 };

#[derive(Parser)]
#[command(name = "phantomarcade", about = "PhantomArcade, a NecroCore app. Serves its page on 127.0.0.1.")]
struct Cli {
    #[arg(long, default_value_t = SPEC.port)]
    port: u16,
    /// NecroCore checkout to read samples from and write .kiro/specs into.
    #[arg(long, default_value = ".")]
    root: PathBuf,
}

fn main() {
    let cli = Cli::parse();
    if let Err(e) = necro_daemon::serve(cli.root, &SPEC, cli.port) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
