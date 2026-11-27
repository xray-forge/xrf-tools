/// What a new game's start created: how many squads and members, and what it could not create, one line each.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SimulationStartReport {
  pub squads: usize,
  pub members: usize,
  pub issues: Vec<String>,
}
