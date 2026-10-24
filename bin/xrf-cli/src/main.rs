#![doc = include_str!("../README.md")]

mod application;
mod commands;
mod core;
mod registry;

use std::process::ExitCode;

use crate::core::logging::setup_logger;

fn main() -> ExitCode {
  setup_logger();
  application::run()
}
