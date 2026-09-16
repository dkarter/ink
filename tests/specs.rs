#![forbid(unsafe_code)]

macro_rules! deferred {
    () => {
        panic!("bootstrap scenario stub: product behavior is not implemented")
    };
}

mod specs {
    mod command_line;
    mod completions_and_performance;
    mod configuration;
    mod placeholders;
    mod presentation;
    mod terminal_io;
    mod themes;
    mod vim_editing;
}
