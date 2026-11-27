/// One thing the simulation tells presentation once, stamped with the tick that stepped it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoopEvent<E> {
  pub tick: u64,
  pub event: E,
}
