/// An object of a spawn that could not be read, by its index among the spawn's objects.
#[derive(Clone, Debug, PartialEq)]
pub struct SpawnSkippedObject {
  pub index: u32,
  pub reason: String,
}
