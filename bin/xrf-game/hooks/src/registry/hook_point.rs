use std::marker::PhantomData;

use crate::registry::hook_kind::HookKind;

/// A hook point a system declares: its name, what it is for, and the function type `H` its entry holds (a `dyn Fn`),
/// so the system reads its entry typed and a script's override by name is checked against it.
pub struct HookPoint<H: ?Sized> {
  pub name: &'static str,
  pub kind: HookKind,
  function: PhantomData<fn() -> Box<H>>,
}

impl<H: ?Sized> HookPoint<H> {
  pub const fn new(name: &'static str, kind: HookKind) -> Self {
    Self {
      name,
      kind,
      function: PhantomData,
    }
  }
}
