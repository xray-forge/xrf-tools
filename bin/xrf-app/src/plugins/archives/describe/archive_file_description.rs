use serde::Serialize;
use xrf_archive::ArchiveReadPolicy;
use xrf_error::{XrfError, XrfResult};
use xrf_extension::XrayExtensionOf;

use crate::plugins::archives::describe::anm::ArchiveAnmDescription;
use crate::plugins::archives::describe::archive_describe_scope::ArchiveDescribeScope;
use crate::plugins::archives::describe::archive_describe_source::ArchiveDescribeSource;
use crate::plugins::archives::describe::archive_described_format::ArchiveDescribedFormat;
use crate::plugins::archives::describe::chunks::ArchiveChunksDescription;
use crate::plugins::archives::describe::detail::{ArchiveDetailLibraryDescription, ArchiveDetailModel};
use crate::plugins::archives::describe::level::{
  ArchiveLevelAiDescription, ArchiveLevelCollisionDescription, ArchiveLevelDescription,
};
use crate::plugins::archives::describe::omf::ArchiveOmfDescription;
use crate::plugins::archives::describe::particles::ArchiveParticlesDescription;
use crate::plugins::archives::describe::ppe::ArchivePpeDescription;
use crate::plugins::archives::describe::shaders::ArchiveShadersDescription;
use crate::plugins::archives::describe::spawn::ArchiveSpawnDescription;
use crate::plugins::archives::describe::thm::ArchiveThmDescription;

/// What the explorer can say about one entry it cannot draw.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum ArchiveFormatDescription {
  // Boxed rather than inline: a description is large beside a refusal, and the refusal is the commoner answer by a
  // wide margin. A line comment because a variant's doc comment travels onto the generated TypeScript member, where
  // a note about Rust layout says nothing.
  Chunks {
    description: Box<ArchiveChunksDescription>,
  },
  Spawn {
    description: Box<ArchiveSpawnDescription>,
  },
  Anm {
    description: Box<ArchiveAnmDescription>,
  },
  Detail {
    description: Box<ArchiveDetailModel>,
  },
  DetailLibrary {
    description: Box<ArchiveDetailLibraryDescription>,
  },
  Level {
    description: Box<ArchiveLevelDescription>,
  },
  LevelAi {
    description: Box<ArchiveLevelAiDescription>,
  },
  LevelCollision {
    description: Box<ArchiveLevelCollisionDescription>,
  },
  Omf {
    description: Box<ArchiveOmfDescription>,
  },
  Particles {
    description: Box<ArchiveParticlesDescription>,
  },
  Ppe {
    description: Box<ArchivePpeDescription>,
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
  /// Nothing describes this format yet, and the file is not a container whose shape could be shown instead. The
  /// extension is the authored spelling, empty for a name without one.
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
  /// only then is its size weighed — because the gate is asked before the bytes are fetched, so an enormous entry is
  /// never held in memory to be turned down.
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

  /// What can be said about an entry no describer claimed, which is its container or nothing.
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
      return Ok(ArchiveFormatDescription::refuse(ArchiveDescribeRefusal::TooLarge {
        size,
        maximum: policy.maximum_chunk_tree_size,
      }));
    }

    Ok(match ArchiveChunksDescription::read(source, name)? {
      Some(description) => ArchiveFormatDescription::Chunks {
        description: Box::new(description),
      },
      None => ArchiveFormatDescription::refuse(ArchiveDescribeRefusal::NoDescriber {
        extension: XrayExtensionOf::of(name).as_str().unwrap_or_default().to_owned(),
      }),
    })
  }
}

impl ArchiveFormatDescription {
  /// The description that says nothing was described, and why.
  fn refuse(reason: ArchiveDescribeRefusal) -> Self {
    Self::Unsupported { reason }
  }
}
