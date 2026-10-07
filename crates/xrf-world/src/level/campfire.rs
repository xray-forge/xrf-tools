/// `CZoneCampfire`'s switching: lit or out, and the turn between them, over which its idle effect starts and stops
/// late (`PlayIdleParticles` and `StopIdleParticles`) and its light ramps (`OVL_TIME`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Campfire {
  is_lit: bool,
  /// When the turn under way ends, in the viewer's milliseconds (`m_turn_time`); none once it has.
  turn_end: Option<u64>,
}

impl Campfire {
  /// `OVL_TIME`: milliseconds a turn takes.
  pub const TURN_TIME: u64 = 3000;

  /// Milliseconds into a turn before the idle effect may play, which stops the enabling one: `OVL_TIME - 2000` left.
  const IDLE_PLAY_DELAY: u64 = 2000;

  /// Milliseconds into a turn before the idle effect may stop: `OVL_TIME - 500` left.
  const IDLE_STOP_DELAY: u64 = 500;

  /// One settled in a state, as a campfire spawns lit (`m_turned_on`) or stands once a script has put it out.
  pub fn new(is_lit: bool) -> Self {
    Self { is_lit, turn_end: None }
  }

  pub fn is_lit(&self) -> bool {
    self.is_lit
  }

  /// `turn_on_script` and `turn_off_script`: starts the turn to a state.
  pub fn turn(&mut self, is_lit: bool, now: u64) {
    self.is_lit = is_lit;
    self.turn_end = Some(now + Self::TURN_TIME);
  }

  /// `UpdateWorkload`: ends the turn once its time is up.
  pub fn settle(&mut self, now: u64) {
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
  pub fn get_light_share(&self, now: u64) -> f32 {
    let left: f32 = self
      .turn_end
      .map_or(0.0, |end| end.saturating_sub(now) as f32 / Self::TURN_TIME as f32);

    if self.is_lit { 1.0 - left } else { left }
  }

  /// `CZoneCampfire::PlayIdleParticles`' gate: settled, or within the turn's last second.
  pub fn can_play_idle(&self, now: u64) -> bool {
    self.is_turned_past(now, Self::IDLE_PLAY_DELAY)
  }

  /// `CZoneCampfire::StopIdleParticles`' gate: settled, or half a second into the turn.
  pub fn can_stop_idle(&self, now: u64) -> bool {
    self.is_turned_past(now, Self::IDLE_STOP_DELAY)
  }

  /// Whether no turn is under way, or more than a delay of it has passed.
  fn is_turned_past(&self, now: u64, delay: u64) -> bool {
    self
      .turn_end
      .is_none_or(|end| end.saturating_sub(now) < Self::TURN_TIME - delay)
  }
}

#[cfg(test)]
mod tests {
  use super::Campfire;

  #[test]
  fn plays_and_stops_its_idle_effect_freely_while_settled() {
    let campfire: Campfire = Campfire::new(true);

    assert!(campfire.is_lit());
    assert!(campfire.can_play_idle(0));
    assert!(campfire.can_stop_idle(0));
  }

  #[test]
  fn plays_its_idle_effect_two_seconds_into_a_turn_on() {
    let mut campfire: Campfire = Campfire::new(false);

    campfire.turn(true, 1000);

    assert!(campfire.is_lit());
    assert!(!campfire.can_play_idle(1000));
    assert!(!campfire.can_play_idle(3000));
    assert!(campfire.can_play_idle(3001));
  }

  #[test]
  fn stops_its_idle_effect_half_a_second_into_a_turn_off() {
    let mut campfire: Campfire = Campfire::new(true);

    campfire.turn(false, 1000);

    assert!(!campfire.is_lit());
    assert!(!campfire.can_stop_idle(1000));
    assert!(!campfire.can_stop_idle(1500));
    assert!(campfire.can_stop_idle(1501));
  }

  #[test]
  fn ramps_its_light_over_a_turn() {
    let mut campfire: Campfire = Campfire::new(true);

    assert_eq!(campfire.get_light_share(0), 1.0);

    campfire.turn(false, 1000);

    assert_eq!(campfire.get_light_share(1000), 1.0);
    assert_eq!(campfire.get_light_share(2500), 0.5);

    campfire.settle(4000);

    assert_eq!(campfire.get_light_share(4000), 0.0);

    campfire.turn(true, 5000);

    assert_eq!(campfire.get_light_share(5000), 0.0);
    assert_eq!(campfire.get_light_share(6500), 0.5);
  }

  #[test]
  fn settles_once_the_turn_has_taken_its_time() {
    let mut campfire: Campfire = Campfire::new(true);

    campfire.turn(false, 1000);
    campfire.settle(3999);

    assert_ne!(campfire, Campfire::new(false));

    campfire.settle(4000);

    assert_eq!(campfire, Campfire::new(false));
  }
}
