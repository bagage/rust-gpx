extern crate clap;

use std::env;
use clap::Shell;

include!("src/cli.rs");

fn main() {
    let outdir = match env::var_os("OUT_DIR") {
        None => return,
        Some(outdir) => outdir,
    };
    let mut cli = build_cli();
    cli.gen_completions("gpxanalyzer", Shell::Bash, &outdir);
    cli.gen_completions("gpxanalyzer", Shell::Fish, &outdir);
    cli.gen_completions("gpxanalyzer", Shell::Zsh, &outdir);
}