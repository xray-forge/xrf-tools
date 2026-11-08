// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![doc = include_str!("../README.md")]

mod application;
mod application_failure;
mod ipc;
mod plugins;
mod window;

use std::process::ExitCode;

use xrf_app_core::logging::ProcessStart;

fn main() -> ExitCode {
  xrf_profiler::start();

  ProcessStart::stamp();

  application::run()
}
