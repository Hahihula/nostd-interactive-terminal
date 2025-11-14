#![no_std]
#![doc = include_str!("../README.md")]

//! An interactive terminal library for `no_std` embedded systems.
//!
//! This crate provides line editing, command history, and command parsing
//! capabilities for embedded systems using async I/O.

pub mod terminal;
pub mod history;
pub mod parser;
pub mod writer;

pub use terminal::{Terminal, TerminalConfig};
pub use history::{History, HistoryConfig};
pub use parser::{CommandParser, ParsedCommand};
pub use writer::TerminalWriter;

/// Re-export commonly used types
pub mod prelude {
    pub use crate::terminal::{Terminal, TerminalConfig};
    pub use crate::history::History;
    pub use crate::parser::{CommandParser, ParsedCommand};
    pub use crate::writer::TerminalWriter;
}