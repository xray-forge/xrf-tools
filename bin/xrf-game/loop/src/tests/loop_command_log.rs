use crate::record::loop_command_log::LoopCommandLog;

#[test]
fn hands_each_tick_its_commands_in_the_order_they_were_taken() {
  let mut log: LoopCommandLog<&str> = LoopCommandLog::default();

  log.push(2, "first").expect("in order");
  log.push(2, "second").expect("in order");
  log.push(5, "third").expect("in order");

  assert_eq!(log.len(), 3);
  assert_eq!(log.take_at(0).count(), 0);
  assert_eq!(log.take_at(2).collect::<Vec<&str>>(), vec!["first", "second"]);
  assert_eq!(log.take_at(3).count(), 0);
  // A tick past a command's takes it rather than leaving it behind.
  assert_eq!(log.take_at(7).collect::<Vec<&str>>(), vec!["third"]);
  assert!(log.is_empty());
}

#[test]
fn ends_past_its_last_command_or_where_its_session_stopped() {
  let mut log: LoopCommandLog<u8> = LoopCommandLog::default();

  assert_eq!(log.get_end(), 0);

  log.push(4, 1).expect("in order");
  assert_eq!(log.get_end(), 5);

  // The session ran on without commands.
  log.end_at(40);
  assert_eq!(log.get_end(), 40);

  // An end before a command it holds is not one.
  log.end_at(3);
  assert_eq!(log.get_end(), 40);
}

#[test]
fn refuses_a_command_of_a_tick_before_the_last() {
  let mut log: LoopCommandLog<u8> = LoopCommandLog::default();

  log.push(4, 1).expect("in order");

  assert!(log.push(3, 2).is_err());
  assert_eq!(log.len(), 1);
}
