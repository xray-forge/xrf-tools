//! What the project contains, and which role each config plays in it.

pub(crate) mod ltx_inventory;
pub(crate) mod ltx_inventory_reader;

pub use crate::inventory::ltx_inventory::{LtxInventory, LtxInventoryFile, LtxInventoryRole};
pub use crate::inventory::ltx_inventory_reader::LtxInventoryReader;
