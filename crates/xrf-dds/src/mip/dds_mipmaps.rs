use crate::mip::dds_mip_filter::DdsMipFilter;

/// What a texture carries below its base level.
///
/// The crate's own policy rather than `image_dds::Mipmaps`, whose four variants describe that encoder's generator: one
/// of them means "take the levels the caller made", which is not a policy a caller states but the mechanism by which
/// [`Self::Filtered`] reaches the encoder at all.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DdsMipmaps {
  /// The base level alone, which is what every UI sheet in this workspace is written with.
  Disabled,
  /// A full chain, reduced with one of the X-Ray converter's kernels.
  Filtered(DdsMipFilter),
}
