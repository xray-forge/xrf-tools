use std::collections::BTreeSet;

use xrf_ltx::{Condlist, CondlistBranch, CondlistCondition};
use xrf_random::RandomStream;

/// What the simulation's condlists are judged against (`xr_logic.pick_section_from_condlist`): the info portions the
/// actor has, none in a new game. A condition function (`=name(...)`, `!name(...)`) answers false, as none is
/// implemented yet; the configs log which their condlists call.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SimulationFacts {
  pub infos: BTreeSet<String>,
}

impl SimulationFacts {
  /// The result of the first branch of `condlist` whose conditions all hold, none where no branch's do or the branch
  /// that holds names none. A probability (`~50`) holds while one draw from `random`, made at the first probability of
  /// the pick and shared by every later one, is at most it, as `pick_section_from_condlist` draws `math.random(100)`
  /// once. A branch's effects (`%...%`) are not run.
  pub fn pick<'a>(&self, condlist: &'a Condlist, random: &mut RandomStream) -> Option<&'a str> {
    let mut draw: Option<u64> = None;

    condlist
      .branches
      .iter()
      .find(|branch| self.holds(branch, random, &mut draw))
      .and_then(|branch| branch.result.as_deref())
  }

  fn holds(&self, branch: &CondlistBranch, random: &mut RandomStream, draw: &mut Option<u64>) -> bool {
    branch.conditions.iter().all(|condition| match condition {
      CondlistCondition::InfoPortion { name, required, .. } => self.infos.contains(name) == *required,
      CondlistCondition::Probability { value, .. } => {
        let rolled: u64 = *draw.get_or_insert_with(|| random.below(100) + 1);

        *value >= rolled as f64
      }
      CondlistCondition::Function { expected, .. } => !*expected,
    })
  }
}
