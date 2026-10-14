/// The module enclosing the one `module_path!()` names at `path`: `path` without its last segment, itself at a root.
pub const fn parent_module_path(path: &'static str) -> &'static str {
  let bytes: &[u8] = path.as_bytes();
  let mut end: usize = bytes.len();

  while end >= 2 {
    if bytes[end - 2] == b':' && bytes[end - 1] == b':' {
      return path.split_at(end - 2).0;
    }

    end -= 1;
  }

  path
}

#[cfg(test)]
mod tests {
  use crate::parent_module_path;

  #[test]
  fn drops_the_last_segment_and_keeps_a_root() {
    assert_eq!(
      parent_module_path("xrf_renderer::contract::limits"),
      "xrf_renderer::contract"
    );
    assert_eq!(parent_module_path("xrf_renderer"), "xrf_renderer");
  }
}
