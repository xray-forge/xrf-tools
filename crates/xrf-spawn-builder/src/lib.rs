#![doc = include_str!("../README.md")]

mod entity;
mod gamedata;
mod graph;
mod map;
mod merge;
mod root;
mod separator;

#[cfg(test)]
mod tests;

pub use crate::gamedata::spawn_listed_level::SpawnListedLevel;
pub use crate::gamedata::spawn_root_gamedata::SpawnRootGamedata;
pub use crate::gamedata::spawn_root_outcome::SpawnRootOutcome;
pub use crate::gamedata::spawn_root_request::SpawnRootRequest;
pub use crate::root::spawn_root_options::SpawnRootOptions;
