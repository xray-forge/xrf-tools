pub(crate) mod dialog_project_opening;
pub(crate) mod find;
pub(crate) mod info;
pub(crate) mod inspect;
pub(crate) mod list;

mod dialog_group;
#[cfg(test)]
pub(crate) mod fixtures;

pub(crate) use dialog_group::DialogGroup;
