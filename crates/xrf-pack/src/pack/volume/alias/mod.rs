//! Payloads one volume already holds, so a later identical file costs a descriptor row and nothing else.

mod archive_alias;
mod archive_alias_candidate;
mod archive_alias_table;

pub(crate) use archive_alias::ArchiveAlias;
pub(crate) use archive_alias_candidate::ArchiveAliasCandidate;
pub(crate) use archive_alias_table::ArchiveAliasTable;
