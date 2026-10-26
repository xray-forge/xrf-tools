/// When a group effect spawns a child effect (`CPGDef::SEffect`): as the effect starts playing, at each birth of one of
/// its particles, or at each death of one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParticleGroupChild {
  /// `m_OnPlayChildName`: one child following each live particle, started as the effect plays.
  Play,
  /// `m_OnBirthChildName`: a free child started at each particle's birth.
  Birth,
  /// `m_OnDeadChildName`: a free child started at each particle's death.
  Death,
}
