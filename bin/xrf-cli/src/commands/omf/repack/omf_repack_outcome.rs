/// How the round trip of one omf file came out.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OmfRepackOutcome {
  Identical,
  Mismatched,
  Failed,
}
