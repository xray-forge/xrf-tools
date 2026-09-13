// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod application;
mod core;
mod ipc;
mod plugins;

use crate::core::logging::setup_logger;
use crate::core::process::setup_process_start;

fn main() {
  setup_process_start();
  setup_logger();

  application::run();
}
