//! The info portions check: where a tree declares, gives, takes and reads each info portion, and the portions those
//! uses do not hold together on.

pub(crate) mod info_portion_mention;
pub(crate) mod info_portion_place;
pub(crate) mod info_portion_role;
pub(crate) mod info_portion_spelling;
pub(crate) mod info_portion_table;
pub(crate) mod info_portion_uses;
pub(crate) mod info_portions_verifier;
pub(crate) mod verify_info_portions_result;

#[cfg(test)]
mod tests;
