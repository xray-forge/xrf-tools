use serde::Serialize;
use xrf_archive::ArchiveReadPolicy;
use xrf_error::{XrfError, XrfResult};
use xrf_extension::XrayExtensionOf;

use crate::plugins::archives::describe::archive_describe_scope::ArchiveDescribeScope;
use crate::plugins::archives::describe::archive_describe_source::ArchiveDescribeSource;
use crate::plugins::archives::describe::archive_described_format::ArchiveDescribedFormat;
use crate::plugins::archives::describe::thm::ArchiveThmDescription;

/// What the explorer can say about one entry it cannot draw.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum ArchiveFormatDescription {
  /// Boxed because a description is large beside a refusal, and the refusal is the commoner answer by a wide margin.
  Thm {
    description: Box<ArchiveThmDescription>,
  },
  Unsupported {
    reason: ArchiveDescribeRefusal,
  },
}

/// Why an entry was not described.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum ArchiveDescribeRefusal {
  /// Nothing describes this format yet.
  NoDescriber { extension: String },
  /// The entry is larger than the policy admits for a read that holds the payload.
  TooLarge { size: u64, maximum: u32 },
}

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
  pub fn of(source: &ArchiveDescribeSource, name: &str, policy: &ArchiveReadPolicy) -> XrfResult<Self> {
    let size: u64 = source
      .get_size_of(name)
      .ok_or_else(|| XrfError::new_not_found_error(format!("File '{name}' is not held by the open subject")))?;

    Ok(Self {
      scope: source.get_scope(),
      format: describe_format(source, name, policy, size)?,
    })
  }
}

fn describe_format(
  source: &ArchiveDescribeSource,
  name: &str,
  policy: &ArchiveReadPolicy,
  size: u64,
) -> XrfResult<ArchiveFormatDescription> {
  let Some(format) = ArchiveDescribedFormat::of(name) else {
    return Ok(ArchiveFormatDescription::Unsupported {
      reason: ArchiveDescribeRefusal::NoDescriber {
        extension: XrayExtensionOf::of(name).as_str().unwrap_or_default().to_owned(),
      },
    });
  };

  // Narrowed rather than compared as a `u64`, because an entry the format cannot describe the size of is past every
  // ceiling this policy sets anyway.
  if !policy.allows_describe_read(u32::try_from(size).unwrap_or(u32::MAX)) {
    return Ok(ArchiveFormatDescription::Unsupported {
      reason: ArchiveDescribeRefusal::TooLarge {
        size,
        maximum: policy.maximum_describe_size,
      },
    });
  }

  match format {
    ArchiveDescribedFormat::Thm => Ok(ArchiveFormatDescription::Thm {
      description: Box::new(ArchiveThmDescription::read(source, name)?),
    }),
  }
}
