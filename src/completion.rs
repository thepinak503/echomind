use clap::Command;
use clap_complete::{generate, Shell};
use std::io;

pub fn generate_completion(shell: Shell) {
    let mut cmd = Command::new("echomind")
        .about("Send piped input to AI chat API and print response")
        .version(env!("CARGO_PKG_VERSION"));

    // Generate the completion script
    generate(shell, &mut cmd, "echomind", &mut io::stdout());
}

#[allow(dead_code)]
pub fn generate_all_completions() -> Vec<(Shell, String)> {
    let shells = vec![
        (Shell::Bash, "bash"),
        (Shell::Zsh, "zsh"),
        (Shell::Fish, "fish"),
        (Shell::PowerShell, "powershell"),
        (Shell::Elvish, "elvish"),
    ];

    shells
        .into_iter()
        .map(|(shell, _name)| {
            let mut buf = Vec::new();
            let mut cmd = Command::new("echomind")
                .about("Send piped input to AI chat API and print response")
                .version(env!("CARGO_PKG_VERSION"));
            generate(shell, &mut cmd, "echomind", &mut buf);
            (shell, String::from_utf8(buf).unwrap())
        })
        .collect()
}
