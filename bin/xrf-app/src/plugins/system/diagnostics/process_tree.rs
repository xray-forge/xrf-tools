//! The processes below one process, and what they cost together.

use serde::Serialize;
use sysinfo::{Pid, Process, System};

/// What everything below one process costs, folded in a single walk of the table.
///
/// A pair rather than one figure, because the total is unreadable without it: a webview runs a browser process, a GPU
/// process and a renderer per frame tree, and "580 MB" means something different across two of those than across
/// eight.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DescendantUsage {
  /// Resident set of every process descended from the root, at any depth.
  pub resident_memory: u64,
  /// How many such processes there are. Zero on a platform that runs the webview in the host process.
  pub processes: u32,
}

/// What every process descended from `root` costs, at any depth.
///
/// Climbs from each process towards `root` rather than descending from it, because the table is keyed by process and
/// holds no children. That is one lookup per link rather than an index nobody else needs.
pub fn sum_descendants(system: &System, root: Pid) -> DescendantUsage {
  let population: usize = system.processes().len();
  let mut usage: DescendantUsage = DescendantUsage::default();

  for (pid, process) in system.processes() {
    if *pid != root && descends_from(process.parent(), root, population, |it| parent_of(system, it)) {
      usage.resident_memory += process.memory();
      usage.processes += 1;
    }
  }

  usage
}

fn parent_of(system: &System, pid: Pid) -> Option<Pid> {
  system.process(pid).and_then(Process::parent)
}

/// Whether `root` is somewhere above `parent`, climbing at most `limit` links.
///
/// Takes the lookup rather than the table so the climb can be tested against a chain nobody has to spawn. `limit` is
/// the number of processes in that table, which bounds the walk: a table read while processes are starting and exiting
/// can name a parent that has already been replaced, and a cycle among those readings would otherwise spin forever.
fn descends_from(parent: Option<Pid>, root: Pid, limit: usize, mut parent_of: impl FnMut(Pid) -> Option<Pid>) -> bool {
  let mut parent: Option<Pid> = parent;

  for _ in 0..limit {
    match parent {
      Some(pid) if pid == root => return true,
      Some(pid) => parent = parent_of(pid),
      None => return false,
    }
  }

  false
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;

  use sysinfo::Pid;

  use super::descends_from;

  fn chain(links: &[(usize, usize)]) -> HashMap<Pid, Pid> {
    links
      .iter()
      .map(|(child, parent)| (Pid::from(*child), Pid::from(*parent)))
      .collect()
  }

  #[test]
  fn finds_a_root_several_links_above() {
    let parents: HashMap<Pid, Pid> = chain(&[(4, 3), (3, 2), (2, 1)]);

    // A webview renderer sits exactly here: two levels under the application, below the browser process.
    assert!(descends_from(
      Some(Pid::from(4)),
      Pid::from(1),
      parents.len() + 1,
      |it| { parents.get(&it).copied() }
    ));
  }

  #[test]
  fn stops_at_a_process_nothing_owns() {
    let parents: HashMap<Pid, Pid> = chain(&[(4, 3)]);

    assert!(!descends_from(
      Some(Pid::from(4)),
      Pid::from(1),
      parents.len() + 1,
      |it| { parents.get(&it).copied() }
    ));
  }

  /// The reason the climb is bounded: a table read while processes come and go can describe a loop, and an unbounded
  /// walk over one would never answer.
  #[test]
  fn refuses_to_climb_a_cycle_forever() {
    let parents: HashMap<Pid, Pid> = chain(&[(2, 3), (3, 2)]);

    assert!(!descends_from(Some(Pid::from(2)), Pid::from(1), parents.len(), |it| {
      parents.get(&it).copied()
    }));
  }
}
