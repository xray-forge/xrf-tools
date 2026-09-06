/// Whether a renderer loads a layout.
///
/// Three answers rather than two, because "nobody has read that loader" and "that loader refuses it" are different
/// claims and only one of them is a finding. A tool that folded them together would tell a modder their texture is
/// broken on the strength of work never done.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DdsFormatSupport {
  Supported,
  Unsupported,
  Unverified,
}
