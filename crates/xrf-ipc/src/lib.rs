#![doc = include_str!("../README.md")]

mod ipc_constant;
mod to_value;

pub use inventory;
pub use ipc_constant::IpcConstant;
pub use serde_json;
pub use to_value::to_value;
pub use xrf_ipc_derive::constant;
