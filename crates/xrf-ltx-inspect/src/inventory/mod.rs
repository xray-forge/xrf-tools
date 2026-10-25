//! What the project contains, and which role each config plays in it.

pub(crate) mod ltx_inventory;
pub(crate) mod ltx_inventory_file;
pub(crate) mod ltx_inventory_reader;
pub(crate) mod ltx_inventory_role;

pub use crate::inventory::ltx_inventory::LtxInventory;
pub use crate::inventory::ltx_inventory_file::LtxInventoryFile;
pub use crate::inventory::ltx_inventory_reader::LtxInventoryReader;
pub use crate::inventory::ltx_inventory_role::LtxInventoryRole;
