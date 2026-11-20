use std::sync::atomic::{AtomicUsize, Ordering};

use rayon::ThreadPoolBuilder;

use crate::GamedataProjectVerifyOptions;
use crate::project::gamedata_asset_sweep::GamedataAssetSweep;

/// The sweep the script, texture, sound, particle, mesh and animation checks share starts no item once the job is
/// cancelled, so a check stopped mid-way reads no further assets.
#[test]
fn a_sweep_starts_no_item_after_cancellation() {
  let options: GamedataProjectVerifyOptions = GamedataProjectVerifyOptions::default();
  let visited: AtomicUsize = AtomicUsize::new(0);

  // A single worker makes the stop boundary deterministic.
  let pool = ThreadPoolBuilder::new().num_threads(1).build().expect("test pool");

  pool
    .install(|| {
      GamedataAssetSweep::new(&options).run(&["first", "second", "third"], |_| {
        visited.fetch_add(1, Ordering::Relaxed);
        options.job.cancel();

        Vec::new()
      })
    })
    .expect("the sweep counts its items");

  assert_eq!(
    visited.load(Ordering::Relaxed),
    1,
    "the sweep kept visiting items after cancellation"
  );
}
