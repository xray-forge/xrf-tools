/// The names a visibility-bitmask search's passes are timed by, which say what it yields.
pub struct BitmaskPassNames {
  pub search: &'static str,
  pub accumulate: &'static str,
  pub filter: &'static str,
}
