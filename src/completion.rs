use clap::CommandFactory;
use clap_complete::{generate, Shell};
use std::io;

pub fn generate_completion(shell: Shell) {
    let mut cmd = crate::cli::Args::command();
    generate(shell, &mut cmd, "echomind", &mut io::stdout());
}
