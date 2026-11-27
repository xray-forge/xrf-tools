/// A level of the game graph: its id, which vertices name, its name, and its config section.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameGraphLevel {
  pub id: u8,
  pub name: String,
  pub section: String,
}
