use crate::alloc::SpanWrites;

#[test]
fn coalesces_contiguous_writes_into_one_run() {
  let mut writes: SpanWrites = SpanWrites::default();

  writes.push(8, &[3, 3, 3, 3]);
  writes.push(0, &[1, 1, 1, 1]);
  writes.push(4, &[2, 2, 2, 2]);
  writes.push(32, &[9, 9, 9, 9]);

  assert_eq!(
    writes.take_runs(),
    [(0, vec![1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3]), (32, vec![9, 9, 9, 9])]
  );
  assert!(writes.is_empty());
}

#[test]
fn applies_a_later_write_over_an_earlier_one() {
  let mut writes: SpanWrites = SpanWrites::default();

  writes.push(0, &[1, 1, 1, 1, 1, 1, 1, 1]);
  writes.push(4, &[2, 2, 2, 2]);
  writes.push(0, &[3, 3, 3, 3]);

  assert_eq!(writes.take_runs(), [(0, vec![3, 3, 3, 3, 2, 2, 2, 2])]);
}
