use serde::Serialize;
use xrf_error::{XrfError, XrfResult};
use xrf_extension::XrayExtensionOf;

use crate::plugins::archives::describe::archive_describe_scope::ArchiveDescribeScope;
use crate::plugins::archives::describe::archive_describe_source::ArchiveDescribeSource;
use crate::plugins::archives::describe::archive_described_format::ArchiveDescribedFormat;
use crate::plugins::archives::describe::omf::ArchiveOmfDescription;
use crate::plugins::archives::describe::particles::ArchiveParticlesDescription;
use crate::plugins::archives::describe::shaders::ArchiveShadersDescription;
use crate::plugins::archives::describe::thm::ArchiveThmDescription;

/// What the explorer can say about one entry it cannot draw.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum ArchiveFormatDescription {
  // Boxed rather than inline: a description is large beside a refusal, and the refusal is the commoner answer by a
  // wide margin. A line comment because a variant's doc comment travels onto the generated TypeScript member, where
  // a note about Rust layout says nothing.
  Omf {
    description: Box<ArchiveOmfDescription>,
  },
  Particles {
    description: Box<ArchiveParticlesDescription>,
  },
  Shaders {
    description: Box<ArchiveShadersDescription>,
  },
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
  /// Nothing describes this format yet. The extension is the authored spelling, empty for a name without one.
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
  /// Reads in the order a refusal is cheapest to reach: the entry has to be listed, a describer has to claim it, and
  /// only then is its size weighed — because an entry nothing describes is refused whatever it weighs, and the gate
  /// is asked before the bytes are fetched so an enormous entry is never held in memory to be turned down.
  ///
  /// # Errors
  ///
  /// Returns an error when the subject holds no such file, or when a describer that claimed the entry could not read
  /// it. A format nothing describes, and an entry too large to read whole, are answers rather than failures.
  pub fn of(source: &ArchiveDescribeSource, name: &str) -> XrfResult<Self> {
    let size: u64 = source
      .get_size_of(name)
      .ok_or_else(|| XrfError::new_not_found_error(format!("File '{name}' is not held by the open subject")))?;

    let format: ArchiveFormatDescription = match ArchiveDescribedFormat::of(name) {
      None => ArchiveFormatDescription::refuse(ArchiveDescribeRefusal::NoDescriber {
        extension: XrayExtensionOf::of(name).as_str().unwrap_or_default().to_owned(),
      }),
      Some(format) => match source.get_read_policy() {
        // Narrowed rather than compared as a `u64`: an entry whose size the format cannot describe is past every
        // ceiling this policy sets anyway.
        policy if !policy.allows_describe_read(u32::try_from(size).unwrap_or(u32::MAX)) => {
          ArchiveFormatDescription::refuse(ArchiveDescribeRefusal::TooLarge {
            size,
            maximum: policy.maximum_describe_size,
          })
        }
        _ => format.describe(source, name)?,
      },
    };

    Ok(Self {
      scope: source.get_scope(),
      format,
    })
  }
}

impl ArchiveFormatDescription {
  /// The description that says nothing was described, and why.
  fn refuse(reason: ArchiveDescribeRefusal) -> Self {
    Self::Unsupported { reason }
  }
}
