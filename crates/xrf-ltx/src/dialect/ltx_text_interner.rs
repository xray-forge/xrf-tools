use std::collections::HashSet;
use std::sync::Arc;

use fxhash::FxBuildHasher;

/// One allocation per distinct piece of text, for the span of resolving one root.
///
/// Inheritance already shares what it copies, because a resolved field is an [`Arc`] and a child clones its parent's
/// handle. This closes the other half: the same key name written in two different sections, or in two files the same
/// root includes, arrives as two separate reads and would otherwise be stored twice. On a vanilla tree that is 293,441
/// resolved fields carrying 10,583 distinct key names.
#[derive(Debug, Default)]
pub(crate) struct LtxTextInterner {
  held: HashSet<Arc<str>, FxBuildHasher>,
}

impl LtxTextInterner {
  /// The shared handle for `text`, allocating only the first time it is seen.
  pub(crate) fn intern(&mut self, text: &str) -> Arc<str> {
    if let Some(held) = self.held.get(text) {
      return Arc::clone(held);
    }

    let shared: Arc<str> = Arc::from(text);

    self.held.insert(Arc::clone(&shared));

    shared
  }
}

#[cfg(test)]
mod test {
  use std::sync::Arc;

  use crate::dialect::LtxTextInterner;

  #[test]
  fn answers_one_allocation_for_repeated_text() {
    let mut interner: LtxTextInterner = LtxTextInterner::default();

    let first: Arc<str> = interner.intern("cost");
    let second: Arc<str> = interner.intern("cost");

    assert_eq!(&*first, "cost");
    assert!(Arc::ptr_eq(&first, &second), "repeated text to share one allocation");
  }

  #[test]
  fn keeps_distinct_text_apart() {
    let mut interner: LtxTextInterner = LtxTextInterner::default();

    let cost: Arc<str> = interner.intern("cost");
    let rpm: Arc<str> = interner.intern("rpm");

    assert!(!Arc::ptr_eq(&cost, &rpm));
    assert_eq!(&*rpm, "rpm");
  }

  #[test]
  fn a_handle_outlives_the_interner_that_made_it() {
    let held: Arc<str> = {
      let mut interner: LtxTextInterner = LtxTextInterner::default();

      interner.intern("ammo_class")
    };

    assert_eq!(&*held, "ammo_class");
  }
}
