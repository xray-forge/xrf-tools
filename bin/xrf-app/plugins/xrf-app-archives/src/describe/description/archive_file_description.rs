use serde::Serialize;
use xrf_archive::ArchiveReadPolicy;
use xrf_error::{XrfError, XrfResult};
use xrf_extension::XrayExtensionOf;

use crate::describe::chunks::ArchiveChunksDescription;
use crate::describe::description::{ArchiveDescribeRefusal, ArchiveDescribeScope, ArchiveFormatDescription};
use crate::describe::source::{ArchiveDescribeSource, ArchiveDescribedFormat, ArchiveEntryReader};

/// One described entry, and what the lookups behind it searched.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveFileDescription {
  pub scope: ArchiveDescribeScope,
  pub format: ArchiveFormatDescription,
}

impl ArchiveFileDescription {
  /// Describes one entry of the subject being browsed.
  ///
  /// # Errors
  ///
  /// Returns an error when the subject holds no such file, or when a describer that claimed the entry could not read
  /// it. A format nothing describes, and an entry too large to read whole, are answers rather than failures.
  pub fn of(source: &ArchiveDescribeSource, name: &str) -> XrfResult<Self> {
    let size: u64 = source
      .get_size_of(name)
      .ok_or_else(|| XrfError::new_not_found_error(format!("File '{name}' is not held by the open subject")))?;

    // Narrowed rather than compared as a `u64`: an entry past `u32` is past every ceiling this policy sets anyway.
    let weighed: u32 = u32::try_from(size).unwrap_or(u32::MAX);
    let policy: &ArchiveReadPolicy = source.get_read_policy();

    let format: ArchiveFormatDescription = match ArchiveDescribedFormat::of(name) {
      None => Self::describe_container(source, name, size, weighed)?,
      // The ceiling bounds what a describer holds, and one that seeks holds a chunk header at a time however large
      // the entry is. Where the subject can only hand the entry over whole it is held after all, and the format is
      // what says whether that is a size worth refusing - see `reads_by_seeking`.
      Some(format) if !format.reads_by_seeking() && !policy.allows_describe_read(weighed) => {
        ArchiveFormatDescription::refuse(ArchiveDescribeRefusal::TooLarge {
          size,
          maximum: policy.maximum_describe_size,
        })
      }
      // A claim that did not hold is offered whatever the container walk can say, exactly as an unclaimed
      // entry is: the file is still a file, and its shape is still readable.
      Some(format) => match format.describe(source, name)? {
        Some(description) => description,
        None => Self::describe_container(source, name, size, weighed)?,
      },
    };

    Ok(Self {
      scope: source.get_scope(),
      format,
    })
  }

  /// What can be said about an entry no describer claimed, which is its container or nothing; weighed before it is
  /// opened, since a volume hands an entry over whole.
  ///
  /// # Errors
  ///
  /// Returns an error when the entry's bytes cannot be read.
  fn describe_container(
    source: &ArchiveDescribeSource,
    name: &str,
    size: u64,
    weighed: u32,
  ) -> XrfResult<ArchiveFormatDescription> {
    let policy: &ArchiveReadPolicy = source.get_read_policy();

    if !policy.allows_chunk_tree_read(weighed) {
      return Ok(Self::refuse_chunk_tree(size, policy));
    }

    Ok(Self::describe_container_of(&mut source.open_entry(name)?, name, policy))
  }

  /// The same for an entry a describer already opened and found not to be its format, walked where it was opened
  /// rather than read again.
  pub(crate) fn describe_container_of(
    entry: &mut ArchiveEntryReader,
    name: &str,
    policy: &ArchiveReadPolicy,
  ) -> ArchiveFormatDescription {
    let size: u64 = entry.size();

    if !policy.allows_chunk_tree_read(u32::try_from(size).unwrap_or(u32::MAX)) {
      return Self::refuse_chunk_tree(size, policy);
    }

    match ArchiveChunksDescription::of_entry(entry) {
      Some(description) => description.into(),
      None => ArchiveFormatDescription::refuse(ArchiveDescribeRefusal::NoDescriber {
        extension: XrayExtensionOf::of(name).as_str().unwrap_or_default().to_owned(),
      }),
    }
  }

  /// The refusal of a container walk over an entry past the ceiling a whole walk answers to.
  fn refuse_chunk_tree(size: u64, policy: &ArchiveReadPolicy) -> ArchiveFormatDescription {
    ArchiveFormatDescription::refuse(ArchiveDescribeRefusal::TooLarge {
      size,
      maximum: policy.maximum_chunk_tree_size,
    })
  }
}
