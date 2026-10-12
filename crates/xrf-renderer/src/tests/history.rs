use crate::frame::history::History;

#[test]
fn reads_what_the_last_frame_wrote() {
  let mut history: History<usize> = History::new(|side| side);

  assert_eq!((*history.get_written(), *history.get_read()), (0, 1));
  assert!(!history.is_valid(), "a new history holds no frame");

  history.swap();

  assert_eq!((*history.get_written(), *history.get_read()), (1, 0));
  assert!(history.is_valid());
  assert_eq!(history.get_frame(), 1);

  history.invalidate();

  assert!(!history.is_valid());
  assert_eq!(history.get_frame(), 1, "an invalidation keeps the frames counted");
}

#[test]
fn fits_or_makes_afresh() {
  let mut slot: Option<History<u32>> = None;

  History::fit(&mut slot, |_| true, |_| 4).swap();

  assert!(History::fit(&mut slot, |history| *history.get_read() == 4, |_| 8).is_valid());
  assert!(
    !History::fit(&mut slot, |history| *history.get_read() == 8, |_| 8).is_valid(),
    "a history that no longer fits is made again"
  );
}
