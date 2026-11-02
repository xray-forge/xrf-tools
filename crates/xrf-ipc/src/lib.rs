#![doc = include_str!("../README.md")]

mod ipc_constant;

#[cfg(test)]
mod tests;

pub use inventory;
pub use ipc_constant::IpcConstant;
pub use serde_json;
pub use xrf_ipc_derive::constant;
