use std::num::NonZeroUsize;
use std::thread;

/// The processors this process may run on: what work sized to the machine is sized to.
///
/// `std::thread::available_parallelism` answers the whole machine on Windows whatever the process's affinity, so a
/// process confined to some processors (`start /affinity`, a job object, a tool measuring one part of the machine) would
/// start a worker for every processor it cannot use, all of them taking turns on the ones it can. There the affinity
/// mask is counted instead, wherever it is narrower than the system's. Elsewhere the standard library already follows
/// the affinity (and, on Linux, the cgroup quota).
pub(crate) struct HostProcessors;

impl HostProcessors {
  /// How many workers "the whole machine" means for this process, or one where the host will not say.
  pub(crate) fn count() -> NonZeroUsize {
    let machine: NonZeroUsize = thread::available_parallelism().unwrap_or(NonZeroUsize::MIN);

    #[cfg(windows)]
    if let Some((process, system)) = Self::read_affinity() {
      return Self::confine(machine, process, system);
    }

    machine
  }

  /// `machine` narrowed to the processors the `process` mask allows, where it allows fewer than the `system` mask has.
  ///
  /// Equal masks are a process that is not confined, and leave `machine` as it is: on a host of more than one processor
  /// group the masks describe a single group, and Windows answers both as zero for a process spanning several, so only a
  /// narrower mask is evidence of a confinement.
  #[cfg(any(windows, test))]
  fn confine(machine: NonZeroUsize, process: usize, system: usize) -> NonZeroUsize {
    if process == system {
      return machine;
    }

    NonZeroUsize::new(process.count_ones() as usize).map_or(machine, |allowed| allowed.min(machine))
  }

  /// This process's affinity mask and the system's, or nothing where Windows will not answer.
  #[cfg(windows)]
  #[expect(unsafe_code, reason = "reads the process's affinity through the Win32 API")]
  fn read_affinity() -> Option<(usize, usize)> {
    use windows::Win32::System::Threading::{GetCurrentProcess, GetProcessAffinityMask};

    let mut process: usize = 0;
    let mut system: usize = 0;

    // SAFETY: the pseudo-handle names this process and both out-pointers are valid for the call.
    unsafe { GetProcessAffinityMask(GetCurrentProcess(), &raw mut process, &raw mut system) }.ok()?;

    Some((process, system))
  }
}

#[cfg(test)]
mod tests {
  use std::num::NonZeroUsize;

  use super::HostProcessors;

  fn count(value: usize) -> NonZeroUsize {
    NonZeroUsize::new(value).expect("test counts are not zero")
  }

  #[test]
  fn leaves_an_unconfined_process_the_whole_machine() {
    assert_eq!(
      HostProcessors::confine(count(32), u32::MAX as usize, u32::MAX as usize),
      count(32)
    );
  }

  #[test]
  fn counts_the_processors_a_narrower_mask_allows() {
    assert_eq!(
      HostProcessors::confine(count(32), 0xffff_0000, u32::MAX as usize),
      count(16)
    );
    assert_eq!(
      HostProcessors::confine(count(32), 0x1_0000, u32::MAX as usize),
      count(1)
    );
  }

  /// Windows answers both masks as zero for a process spanning processor groups, which is no confinement.
  #[test]
  fn reads_masks_that_say_nothing_as_no_confinement() {
    assert_eq!(HostProcessors::confine(count(128), 0, 0), count(128));
  }

  #[test]
  fn never_answers_more_than_the_machine() {
    assert_eq!(HostProcessors::confine(count(4), 0xff, 0xffff), count(4));
  }
}
