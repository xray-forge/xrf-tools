use std::path::PathBuf;

use xrf_dds::DdsMetadata;

/// The outcome of resolving and reading one texture reference.
pub enum TextureResolution {
  /// The visual sits under no directory that looks like an X-Ray root.
  NoRoot,
  /// Nothing searched holds the reference. `sources` is every root the search looked in, in order.
  Missing { sources: Vec<String> },
  /// The texture resolved and its DDS header was read.
  Resolved {
    step: String,
    path: PathBuf,
    format: String,
    metadata: DdsMetadata,
  },
  /// The texture resolved inside an archive, which this sweep locates but does not inspect.
  ///
  /// Separate from [`TextureResolution::Resolved`] because the census question splits here: the reference is present,
  /// and its format stays unknown. Reading it would mean decompressing a whole entry to look at 148 bytes of header.
  Located { step: String, container: String },
  /// The texture resolved, but its DDS header could not be read or parsed.
  Unreadable { path: PathBuf, reason: String },
  /// The visual's own root could not be searched, so none of its references could be looked for.
  RootUnreadable { root: PathBuf, reason: String },
  /// The search refused the reference itself, as no path the engine could resolve.
  Rejected { reason: String },
}
