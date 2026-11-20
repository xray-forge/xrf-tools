/// What a fog volume does in its group, `EFogVolume::m_volumeType`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditableFogVolumeKind {
  /// `fvEmitter`, the group's fog body, simulated as its profile, a config under `$game_config$`, sets.
  Emitter { profile: String },
  /// `fvOcclusion`, a body the group's fog flows around.
  Occlusion,
}
