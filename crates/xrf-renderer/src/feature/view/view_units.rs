/// Lists a group of a view's units from one list, so the count and the units never disagree: how many it holds, and
/// each as what every unit answers alike.
macro_rules! view_units {
  ($owner:ty { $($unit:ident),+ $(,)? }) => {
    impl $owner {
      /// How many units it holds.
      pub const UNIT_COUNT: usize = [$(stringify!($unit)),+].len();

      /// Every unit, to let go of or change alike.
      pub fn list_mut(
        &mut self,
      ) -> [&mut dyn $crate::feature::view::view_feature::ViewFeature; Self::UNIT_COUNT] {
        [$(&mut self.$unit),+]
      }
    }
  };
}

pub(crate) use view_units;
