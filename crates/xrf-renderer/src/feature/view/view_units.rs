/// Lists a group of a view's units twice from one list, so neither list names a unit the other leaves out: how many it
/// holds, and each as what every unit answers alike.
macro_rules! view_units {
  ($owner:ty { $($unit:ident),+ $(,)? }) => {
    impl $owner {
      /// How many units it holds.
      pub const UNIT_COUNT: usize = [$(stringify!($unit)),+].len();

      /// Every unit, as what each answers alike.
      pub fn list(&self) -> [&dyn $crate::feature::view::view_feature::ViewFeature; Self::UNIT_COUNT] {
        [$(&self.$unit),+]
      }

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
