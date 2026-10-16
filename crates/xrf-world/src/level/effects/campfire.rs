use std::time::Duration;

/// `CZoneCampfire`'s switching: lit or out, and the turn between them, over which its idle effect starts and stops
/// late (`PlayIdleParticles` and `StopIdleParticles`) and its light ramps (`OVL_TIME`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Campfire {
  is_lit: bool,
  /// When the turn under way ends, on the level's clock (`m_turn_time`); none once it has.
  turn_end: Option<Duration>,
}

impl Campfire {
  /// `OVL_TIME`: how long a turn takes.
  pub const TURN_TIME: Duration = Duration::from_millis(3000);

  /// How far into a turn the idle effect may play, which stops the enabling one: `OVL_TIME - 2000` left.
  const IDLE_PLAY_DELAY: Duration = Duration::from_millis(2000);

  /// How far into a turn the idle effect may stop: `OVL_TIME - 500` left.
  const IDLE_STOP_DELAY: Duration = Duration::from_millis(500);

  /// One settled in a state, as a campfire spawns lit (`m_turned_on`) or stands once a script has put it out.
  pub fn new(is_lit: bool) -> Self {
    Self { is_lit, turn_end: None }
  }

  pub fn is_lit(&self) -> bool {
    self.is_lit
  }

  /// `turn_on_script` and `turn_off_script`: starts the turn to a state.
  pub fn turn(&mut self, is_lit: bool, now: Duration) {
    self.is_lit = is_lit;
    self.turn_end = Some(now + Self::TURN_TIME);
  }

  /// `UpdateWorkload`: ends the turn once its time is up.
  pub fn settle(&mut self, now: Duration) {
    if self.turn_end.is_some_and(|end| end <= now) {
      self.turn_end = None;
    }
  }

  /// Whether a turn is under way.
  pub fn is_turning(&self) -> bool {
    self.turn_end.is_some()
  }

  /// `UpdateWorkload`'s `k`: how much of its idle light shows, ramping up over a turn on and down over a turn off; all
  /// of it while lit, none while out.
  pub fn get_light_share(&self, now: Duration) -> f32 {
    let left: f32 = self.turn_end.map_or(0.0, |end| {
      end.saturating_sub(now).as_secs_f32() / Self::TURN_TIME.as_secs_f32()
    });

    if self.is_lit { 1.0 - left } else { left }
  }

  /// `CZoneCampfire::PlayIdleParticles`' gate: settled, or within the turn's last second.
  pub fn can_play_idle(&self, now: Duration) -> bool {
    self.is_turned_past(now, Self::IDLE_PLAY_DELAY)
  }

  /// `CZoneCampfire::StopIdleParticles`' gate: settled, or half a second into the turn.
  pub fn can_stop_idle(&self, now: Duration) -> bool {
    self.is_turned_past(now, Self::IDLE_STOP_DELAY)
  }

  /// Whether no turn is under way, or more than a delay of it has passed.
  fn is_turned_past(&self, now: Duration, delay: Duration) -> bool {
    self
      .turn_end
      .is_none_or(|end| end.saturating_sub(now) < Self::TURN_TIME - delay)
  }
}
