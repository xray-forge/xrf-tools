// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod application;
mod core;
mod ipc;
mod plugins;

use std::process::ExitCode;

use crate::core::process::setup_process_start;

fn main() -> ExitCode {
  xrf_renderer::start_profiler();
  setup_process_start();

  application::run()
}
