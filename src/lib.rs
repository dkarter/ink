#![forbid(unsafe_code)]

pub mod cli;
pub mod config;
pub mod editor;
#[doc(hidden)]
pub mod startup_bench;

mod terminal;
mod theme;
mod ui;
