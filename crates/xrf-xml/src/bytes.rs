/// Where `needle` first occurs in `haystack`, the byte search every reader of undecoded XML shares.
pub(crate) fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
  haystack.windows(needle.len()).position(|window| window == needle)
}
