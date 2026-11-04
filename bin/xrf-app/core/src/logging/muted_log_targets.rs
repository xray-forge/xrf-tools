/// The log targets the application's log mutes, holding them to errors whatever level the rest logs at, unless
/// `RUST_LOG` names one: dependencies' warnings that tell a user or a developer nothing to act on, each with why.
pub(crate) struct MutedLogTargets;

impl MutedLogTargets {
  /// Symphonia's Ogg reader parses every Vorbis comment, with no option to skip them, and warns of each that is not
  /// `KEY=value`: every game sound's first comment is not, the engine's source parameters in binary, which `xrf-sound`
  /// reads instead.
  pub(crate) const SYMPHONIA_VORBIS_COMMENTS: &'static str = "symphonia_metadata::embedded::vorbis";

  /// Every target muted.
  pub(crate) const ALL: [&'static str; 1] = [Self::SYMPHONIA_VORBIS_COMMENTS];
}
