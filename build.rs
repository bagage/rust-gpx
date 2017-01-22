extern crate clap;

use std::env;
use clap::Shell;
use std::path::Path;

include!("src/cli.rs");

fn main() {
    let outdir = match env::var_os("XDG_DATA_HOME") {
        None => return,
        Some(outdir) => outdir,
    };
    let completiondir = Path::new(&outdir).join("zsh");
    let mut cli = build_cli();
	println!("Will install ZSH completion to {}", completiondir.to_string_lossy());
    cli.gen_completions("gpxanalyzer", Shell::Zsh, &completiondir);
}