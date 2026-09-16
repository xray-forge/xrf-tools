/// The curve a key interpolates along, `SHAPE_*` (`xrCore/Animation/Envelope.hpp`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArchiveAnimationShape {
  Tcb,
  Hermite,
  Bezier,
  Linear,
  Stepped,
  Bezier2,
}

impl ArchiveAnimationShape {
  /// Every shape the engine names, in the order it numbers them.
  pub const NAMED: [Self; 6] = [
    Self::Tcb,
    Self::Hermite,
    Self::Bezier,
    Self::Linear,
    Self::Stepped,
    Self::Bezier2,
  ];

  /// What the shape is called in the terms an author would recognise.
  pub const fn label(self) -> &'static str {
    match self {
      Self::Tcb => "tcb",
      Self::Hermite => "hermite",
      Self::Bezier => "bezier",
      Self::Linear => "linear",
      Self::Stepped => "stepped",
      Self::Bezier2 => "bezier 2",
    }
  }

  /// The number the engine stores this shape as.
  pub const fn value(self) -> u8 {
    self as u8
  }

  /// The shapes a run of keys uses, named, in the order the engine numbers them.
  ///
  /// A shape no name here claims keeps its number instead, so a file written by another tool does not read as using
  /// no shape at all.
  pub fn label_all(shapes: impl IntoIterator<Item = u8>) -> Vec<String> {
    let mut used: Vec<u8> = shapes.into_iter().collect();

    used.sort_unstable();
    used.dedup();

    used
      .into_iter()
      .map(
        |shape| match Self::NAMED.into_iter().find(|named| named.value() == shape) {
          Some(named) => named.label().to_owned(),
          None => format!("shape {shape}"),
        },
      )
      .collect()
  }
}

#[cfg(test)]
mod tests {
  use super::ArchiveAnimationShape;

  #[test]
  fn each_shape_sits_where_the_engine_numbers_it() {
    assert_eq!(ArchiveAnimationShape::Tcb.value(), 0);
    assert_eq!(ArchiveAnimationShape::Stepped.value(), 4);
    assert_eq!(ArchiveAnimationShape::Bezier2.value(), 5);
  }

  #[test]
  fn a_run_of_keys_names_each_shape_it_uses_once() {
    // The mix of a vanilla camera effect: mostly tcb with a few linear segments.
    assert_eq!(
      ArchiveAnimationShape::label_all([0, 3, 0, 0, 3]),
      vec![String::from("tcb"), String::from("linear")]
    );
  }

  #[test]
  fn a_shape_the_engine_does_not_name_keeps_its_number() {
    assert_eq!(ArchiveAnimationShape::label_all([7]), vec![String::from("shape 7")]);
  }
}
