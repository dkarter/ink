//! Command-line parsing and top-level dispatch.

use std::{ffi::OsStr, process::ExitCode};

use crate::{
    app,
    config::{self, CliOptions, ConfigPaths, Settings, ThemeName},
    terminal::{self, ExitStatus, ProcessIo},
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
    Input(InputPrompt),
    /// Edit multiline text.
    Textarea(TextareaPrompt),
    /// Generate a shell completion script.
    Completion(Completion),
}

#[derive(Args)]
struct InputPrompt {
    /// Initial editable text. Takes precedence over piped stdin.
    #[usage(long)]
    value: Option<String>,

    /// Start in Vim Normal mode instead of Insert mode.
    #[usage(long)]
    normal: bool,

    /// Select a bundled or configured theme.
    #[usage(long)]
    theme: Option<String>,

    /// Text shown before an input value. Use an empty value to hide it.
    #[usage(long)]
    prompt: Option<String>,

    /// Guidance shown while the editable value is empty.
    #[usage(long)]
    placeholder: Option<String>,
}

#[derive(Args)]
struct TextareaPrompt {
    /// Initial editable text. Takes precedence over piped stdin.
    #[usage(long)]
    value: Option<String>,

    /// Start in Vim Normal mode instead of Insert mode.
    #[usage(long)]
    normal: bool,

    /// Select a bundled or configured theme.
    #[usage(long)]
    theme: Option<String>,

    /// Use the complete terminal area instead of an inline prompt.
    #[usage(long)]
    fullscreen: bool,

    /// Guidance shown while the editable value is empty.
    #[usage(long)]
    placeholder: Option<String>,
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

/// Prompt values that affect process I/O or presentation.
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromptRuntimeOptions {
    pub value: Option<String>,
    pub prompt: String,
    pub placeholder: String,
    pub fullscreen: bool,
}

/// Runtime effects performed after parsing.
#[doc(hidden)]
pub trait CliRuntime {
    fn write_stdout(&mut self, text: &str);

    /// Enter prompt execution.
    fn run_prompt(
        &mut self,
        kind: PromptKind,
        prompt: PromptRuntimeOptions,
        options: ResolvedPromptOptions,
    ) -> ExitCode;
}

/// Validated settings and palette ready for the prompt runtime.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResolvedPromptOptions {
    pub settings: Settings,
    pub theme: theme::ResolvedTheme,
    pub input_background: bool,
}

/// Resolve all configuration that the prompt runtime will receive.
#[doc(hidden)]
#[must_use]
pub fn resolve_prompt_options(config: &config::Config, cli: &CliOptions) -> ResolvedPromptOptions {
    let settings = Settings::resolve(config, cli);
    let theme = theme::resolve(settings.theme, &config.colors);
    let input_background = config.colors.contains_key(&theme::ColorRole::Background);
    ResolvedPromptOptions {
        settings,
        theme,
        input_background,
    }
}

struct ProcessRuntime;

impl CliRuntime for ProcessRuntime {
    fn write_stdout(&mut self, text: &str) {
        print!("{text}");
    }

    fn run_prompt(
        &mut self,
        kind: PromptKind,
        mut prompt: PromptRuntimeOptions,
        options: ResolvedPromptOptions,
    ) -> ExitCode {
        let value = prompt.value.take();
        terminal::run_prompt(&mut ProcessIo, value, |session, seed| {
            app::run(session, kind, seed, &prompt, options)
        })
        .into()
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
        Command::Input(prompt) => dispatch_prompt(
            prompt.normal,
            prompt.theme,
            PromptKind::Input,
            PromptRuntimeOptions {
                value: prompt.value,
                prompt: prompt.prompt.unwrap_or_default(),
                placeholder: prompt.placeholder.unwrap_or_default(),
                fullscreen: false,
            },
            runtime,
        ),
        Command::Textarea(prompt) => dispatch_prompt(
            prompt.normal,
            prompt.theme,
            PromptKind::Textarea,
            PromptRuntimeOptions {
                value: prompt.value,
                prompt: String::new(),
                placeholder: prompt.placeholder.unwrap_or_default(),
                fullscreen: prompt.fullscreen,
            },
            runtime,
        ),
    }
}

fn dispatch_prompt(
    normal: bool,
    theme: Option<String>,
    kind: PromptKind,
    prompt: PromptRuntimeOptions,
    runtime: &mut impl CliRuntime,
) -> ExitCode {
    let theme = match theme.map(|theme| theme.parse::<ThemeName>()).transpose() {
        Ok(theme) => theme,
        Err(error) => {
            eprintln!("invalid command-line setting `--theme`: {error}");
            return ExitStatus::UsageError.into();
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
        normal: normal.then_some(true),
        theme,
    };
    let resolved_options = resolve_prompt_options(&config, &cli_options);
    runtime.run_prompt(kind, prompt, resolved_options)
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
