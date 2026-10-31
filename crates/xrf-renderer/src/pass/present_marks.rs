/// What the present lays over a frame of the helpers' marks: whether the models they mark are tinted and outlined, and
/// whether what neither a mark nor the selection covers is faded ("Fade the rest").
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PresentMarks {
  pub is_marked: bool,
  pub is_rest_faded: bool,
}
