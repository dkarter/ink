//! Command-line parsing and top-level dispatch.

use std::{ffi::OsStr, process::ExitCode};

use crate::{
    config::{self, CliOptions, ConfigPaths, Settings, ThemeName},
    theme,
};
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

/// Prompt command selected by the static CLI parser.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PromptKind {
    Input,
    Textarea,
}

/// Runtime effects performed after parsing.
#[doc(hidden)]
pub trait CliRuntime {
    fn write_stdout(&mut self, text: &str);

    /// Record an attempt to read prompt input.
    fn read_stdin(&mut self);

    /// Record an attempt to open the controlling terminal.
    fn open_controlling_terminal(&mut self);

    /// Enter prompt execution.
    fn run_prompt(&mut self, kind: PromptKind) -> ExitCode;
}

/// Validated settings and palette ready for the prompt runtime.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResolvedPromptOptions {
    pub settings: Settings,
    pub theme: theme::ResolvedTheme,
}

/// Resolve all configuration that the prompt runtime will receive.
#[doc(hidden)]
#[must_use]
pub fn resolve_prompt_options(config: &config::Config, cli: &CliOptions) -> ResolvedPromptOptions {
    let settings = Settings::resolve(config, cli);
    let theme = theme::resolve(settings.theme, &config.colors);
    ResolvedPromptOptions { settings, theme }
}

struct ProcessRuntime;

impl CliRuntime for ProcessRuntime {
    fn write_stdout(&mut self, text: &str) {
        print!("{text}");
    }

    fn read_stdin(&mut self) {
        unreachable!("prompt input is not implemented")
    }

    fn open_controlling_terminal(&mut self) {
        unreachable!("terminal access is not implemented")
    }

    fn run_prompt(&mut self, _kind: PromptKind) -> ExitCode {
        eprintln!("ink prompts are not implemented in this bootstrap release");
        ExitCode::FAILURE
    }
}

fn dispatch(cli: Ink, runtime: &mut impl CliRuntime) -> ExitCode {
    match cli.command {
        Command::Completion(completion) => {
            let shell = usage::complete::Shell::from_name(&completion.shell)
                .expect("shell choices are validated by usage-rs");
            runtime.write_stdout(&Ink::completion_script(shell));
            ExitCode::SUCCESS
        }
        Command::Input(prompt) => dispatch_prompt(prompt, PromptKind::Input, runtime),
        Command::Textarea(prompt) => dispatch_prompt(prompt, PromptKind::Textarea, runtime),
    }
}

fn dispatch_prompt(prompt: Prompt, kind: PromptKind, runtime: &mut impl CliRuntime) -> ExitCode {
    let theme = match prompt
        .theme
        .map(|theme| theme.parse::<ThemeName>())
        .transpose()
    {
        Ok(theme) => theme,
        Err(error) => {
            eprintln!("invalid command-line setting `--theme`: {error}");
            return ExitCode::FAILURE;
        }
    };
    let config = match config::load(&ConfigPaths::from_env()) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::FAILURE;
        }
    };
    let cli_options = CliOptions {
        normal: prompt.normal.then_some(true),
        theme,
    };
    let resolved_options = resolve_prompt_options(&config, &cli_options);
    let _ = (prompt.value, resolved_options);
    runtime.run_prompt(kind)
}

/// Parse explicit arguments from the compiled CLI tables and dispatch them to a runtime.
#[doc(hidden)]
pub fn run_from(args: &[&OsStr], runtime: &mut impl CliRuntime) -> Result<ExitCode, String> {
    Ink::parse_from(args)
        .map(|cli| dispatch(cli, runtime))
        .map_err(|error| format!("{error:?}"))
}

/// Parse command-line arguments and run the selected command.
#[must_use]
pub fn run() -> ExitCode {
    dispatch(Ink::parse(), &mut ProcessRuntime)
}
