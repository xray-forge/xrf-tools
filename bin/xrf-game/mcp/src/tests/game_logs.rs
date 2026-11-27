use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use crate::bridge::game_logs::GameLogs;

fn directory(name: &str) -> PathBuf {
  let directory: PathBuf = std::env::temp_dir().join(format!("xrf-game-mcp-{name}-{}", std::process::id()));

  let _ = fs::remove_dir_all(&directory);
  fs::create_dir_all(&directory).expect("the directory is made");

  directory
}

#[test]
fn reads_the_tail_of_the_newest_game_log_and_no_other_log() {
  let logs: PathBuf = directory("tail");
  let older: PathBuf = logs.join("xrf-game.2026-11-27T01-00-00Z.1.log");
  let newer: PathBuf = logs.join("xrf-game.2026-11-27T02-00-00Z.2.log");

  fs::write(&older, "old\n").expect("written");
  fs::write(&newer, "one\ntwo\nthree\n").expect("written");
  fs::write(logs.join("xrf-app.2026-11-27T03-00-00Z.3.log"), "the app's\n").expect("written");
  fs::File::options()
    .write(true)
    .open(&older)
    .and_then(|file| file.set_modified(SystemTime::now() - Duration::from_secs(60)))
    .expect("dated");

  let tail: String = GameLogs::new(vec![logs.clone()]).read_tail(2).expect("a log reads");

  assert_eq!(tail, format!("{}\ntwo\nthree", newer.display()));

  let _ = fs::remove_dir_all(&logs);
}

#[test]
fn says_when_the_game_wrote_no_log() {
  let logs: PathBuf = directory("none");

  assert!(GameLogs::new(vec![logs.clone()]).read_tail(10).is_err());

  let _ = fs::remove_dir_all(&logs);
}
