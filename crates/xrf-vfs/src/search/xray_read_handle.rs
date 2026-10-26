/// A file opened to be read in pieces, which [`XraySearch::open`](crate::XraySearch::open) does not hand out yet.
#[derive(Debug)]
pub struct XrayReadHandle {
  _private: (),
}
