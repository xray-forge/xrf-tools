use crate::sound::LevelWorldSounds;

#[test]
fn scans_a_track_as_the_engine_does() {
  assert_eq!(LevelWorldSounds::scan_track("6,22,0.8,30,60"), ([6, 22], 0.8, [30, 60]));
  assert_eq!(
    LevelWorldSounds::scan_track(" 6, 22, 0.8, 30, 60"),
    ([6, 22], 0.8, [30, 60])
  );
  assert_eq!(
    LevelWorldSounds::scan_track("6,22,5e-1,+30,-60"),
    ([6, 22], 0.5, [30, -60])
  );
}

#[test]
fn stops_at_the_first_value_that_does_not_scan_and_keeps_the_defaults_after_it() {
  // `%d` takes the number before the letters, then the next comma does not follow it.
  assert_eq!(
    LevelWorldSounds::scan_track("12abc,22,0.8,30,60"),
    ([12, 0], 1.0, [0, 0])
  );
  // A space before a comma is not skipped: only a number's own conversion skips spaces.
  assert_eq!(LevelWorldSounds::scan_track("6 ,22,0.8,30,60"), ([6, 0], 1.0, [0, 0]));
  assert_eq!(LevelWorldSounds::scan_track("6,22,loud,30,60"), ([6, 22], 1.0, [0, 0]));
  assert_eq!(LevelWorldSounds::scan_track("6,22,.,30,60"), ([6, 22], 1.0, [0, 0]));
  assert_eq!(LevelWorldSounds::scan_track(""), ([0, 0], 1.0, [0, 0]));
}

#[test]
fn takes_the_longest_number_each_conversion_accepts() {
  // An `e` with no digit after it ends the float before it, which leaves `e` where the comma should be.
  assert_eq!(LevelWorldSounds::scan_track("6,22,2e,30,60"), ([6, 22], 2.0, [0, 0]));
  assert_eq!(LevelWorldSounds::scan_track("6,22,.5,30,60"), ([6, 22], 0.5, [30, 60]));
  assert_eq!(LevelWorldSounds::scan_track("6,22,3.,30,60x"), ([6, 22], 3.0, [30, 60]));
  assert_eq!(
    LevelWorldSounds::scan_track("99999999999999999999,22,1,30,60"),
    ([i64::MAX, 22], 1.0, [30, 60])
  );
}
