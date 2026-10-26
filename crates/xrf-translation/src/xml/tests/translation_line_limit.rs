use crate::xml::translation_line_limit::TranslationLineLimit;

#[test]
fn a_line_at_the_limit_loads_and_one_past_it_does_not() {
  let at: Vec<u8> = vec![b'a'; TranslationLineLimit::MAX_BYTES];
  let past: Vec<u8> = vec![b'a'; TranslationLineLimit::MAX_BYTES + 1];

  assert!(TranslationLineLimit::find_overlong(&at).is_empty());
  assert_eq!(TranslationLineLimit::find_overlong(&past), vec![(1, past.as_slice())]);
}

#[test]
fn line_breaks_do_not_count_and_lines_are_numbered_by_line_feeds() {
  let mut bytes: Vec<u8> = b"<string_table>\r\n".to_vec();

  bytes.extend(vec![b'a'; TranslationLineLimit::MAX_BYTES]);
  bytes.extend(b"\r\n");
  bytes.extend(vec![b'b'; TranslationLineLimit::MAX_BYTES + 1]);
  bytes.extend(b"\r\n</string_table>\r\n");

  let overlong: Vec<(usize, &[u8])> = TranslationLineLimit::find_overlong(&bytes);

  assert_eq!(overlong.len(), 1);
  assert_eq!(overlong[0].0, 3);
  assert_eq!(overlong[0].1.len(), TranslationLineLimit::MAX_BYTES + 1);
}

#[test]
fn a_carriage_return_ends_a_line_as_the_engine_reads_it() {
  // Two halves under the limit, joined only by a lone carriage return, are two lines to the engine's reader.
  let mut bytes: Vec<u8> = vec![b'a'; 3000];

  bytes.push(b'\r');
  bytes.extend(vec![b'b'; 3000]);

  assert!(TranslationLineLimit::find_overlong(&bytes).is_empty());
}
