//! Command-line parsing and top-level dispatch.

use std::process::ExitCode;

use usage::{Args, Cli, Subcommands};

#[derive(Cli)]
#[usage(
    bin = "ink",
    version = env!("CARGO_PKG_VERSION"),
    completion,
    about = "A fast, composable terminal input prompt with Vim editing"
)]
struct Ink {
    #[usage(subcommand)]
    command: Command,
}

#[derive(Subcommands)]
enum Command {
    /// Edit a single logical line of text.
    Input(Prompt),
    /// Edit multiline text.
    Textarea(Prompt),
    /// Generate a shell completion script.
    Completion(Completion),
}

#[derive(Args)]
struct Prompt {
    /// Initial editable text. Takes precedence over piped stdin.
    #[usage(long)]
    value: Option<String>,

    /// Start in Vim Normal mode instead of Insert mode.
    #[usage(long)]
    normal: bool,

    /// Select a bundled or configured theme.
    #[usage(long)]
    theme: Option<String>,
}

#[derive(Args)]
struct Completion {
    /// Shell to generate completions for.
    #[usage(choices("bash", "zsh", "fish", "nu"))]
    shell: String,
}

/// Parse command-line arguments and run the selected command.
#[must_use]
pub fn run() -> ExitCode {
    match Ink::parse().command {
        Command::Completion(completion) => {
            let shell = usage::complete::Shell::from_name(&completion.shell)
                .expect("shell choices are validated by usage-rs");
            print!("{}", Ink::completion_script(shell));
            ExitCode::SUCCESS
        }
        Command::Input(prompt) | Command::Textarea(prompt) => {
            let _ = (prompt.value, prompt.normal, prompt.theme);
            eprintln!("ink prompts are not implemented in this bootstrap release");
            ExitCode::FAILURE
        }
    }
}
