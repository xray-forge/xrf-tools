#![allow(dead_code)]

pub const NIL: &str = "nil";

pub const MINIMAL_SUPPORTED_SPAWN_VERSION: u16 = 120;

pub const FLAG_SPAWN_ENABLED: u16 = 1;
pub const FLAG_SPAWN_ON_SURGE_ONLY: u16 = 2;
pub const FLAG_SPAWN_SINGLE_ITEM_ONLY: u16 = 4;
pub const FLAG_SPAWN_IF_DESTROYED_ONLY: u16 = 8;
pub const FLAG_SPAWN_INFINITE_COUNT: u16 = 16;
pub const FLAG_SPAWN_DESTROY_ON_SPAWN: u16 = 32;

pub const FLAG_SKELETON_ACTIVE: u8 = 1;
pub const FLAG_SKELETON_SPAWN_COPY: u8 = 2;
pub const FLAG_SKELETON_SAVED_DATA: u8 = 4;
pub const FLAG_SKELETON_NOT_SAVE: u8 = 8;

// Graph related chunk sizing.
pub const DEFAULT_LEVEL_BLOCK_SIZE: usize = 4096;
pub const DEFAULT_VERTEX_BLOCK_SIZE: usize = 42;
pub const DEFAULT_EDGE_BLOCK_SIZE: usize = 6;
pub const DEFAULT_POINT_BLOCK_SIZE: usize = 20;

pub const NET_ACTION_UPDATE: u16 = 0;
pub const NET_ACTION_SPAWN: u16 = 1;
