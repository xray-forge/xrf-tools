/// A squad a new game starts with: its section and the smart terrain it is made at and enters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StartSquad {
  pub squad: String,
  pub smart: String,
}
