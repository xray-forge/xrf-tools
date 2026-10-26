//! How selected entries become volumes: the layout a cap allows, the tables a volume carries, and the writer.
//!
//! [`ArchivePublishedSet`] is the other direction — what the destination already holds under the run's own name,
//! which is both what a run refuses to overwrite and what a failed run takes back.
//!
//! - [`alias`] — payloads already written to a volume, which an identical file points at instead of repeating.

mod alias;
mod archive_descriptor_table;
mod archive_published_set;
mod archive_volume_layout;
mod archive_volume_writer;
mod descriptor_name;

pub(crate) use alias::{ArchiveAlias, ArchiveAliasCandidate, ArchiveAliasTable};
pub(crate) use archive_descriptor_table::ArchiveDescriptorTable;
pub(crate) use archive_published_set::ArchivePublishedSet;
pub(crate) use archive_volume_layout::ArchiveVolumeLayout;
pub(crate) use archive_volume_writer::ArchiveVolumeWriter;
pub(crate) use descriptor_name::DescriptorName;
