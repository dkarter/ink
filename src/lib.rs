#![forbid(unsafe_code)]

mod app;
pub mod cli;
pub mod config;
mod display;
pub mod editor;
#[doc(hidden)]
pub mod startup_bench;

#[doc(hidden)]
pub mod terminal;
mod text;
pub mod theme;
pub mod ui;
