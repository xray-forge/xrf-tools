use std::fmt::Display;
use std::fs;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use rayon::ThreadPoolBuilder;
use xrf_job::{JobHandle, JobOutcome};
use xrf_output::{Output, OutputChannel, OutputOptions, OutputVerbosity};
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

use crate::{
  GamedataProject, GamedataProjectReadOptions, GamedataProjectVerifyOptions, GamedataVerificationStatus,
  GamedataVerificationType,
};

struct CancelAfterFirstItem {
  job: JobHandle,
  prefix: &'static str,
  items: AtomicUsize,
}

impl Output for CancelAfterFirstItem {
  fn write(&self, _: OutputChannel, message: &dyn Display) {
    if message.to_string().starts_with(self.prefix) {
      self.items.fetch_add(1, Ordering::Relaxed);
      self.job.cancel();
    }
  }
}

#[test]
fn cancellation_inside_a_check_stops_its_remaining_items() {
  for (check, directory, extension, contents, prefix) in [
    (
      GamedataVerificationType::Scripts,
      "scripts",
      "script",
      "local a = 1",
      "Verify script:",
    ),
    (
      GamedataVerificationType::Textures,
      "textures",
      "dds",
      "invalid",
      "Verify texture:",
    ),
    (
      GamedataVerificationType::Sounds,
      "sounds",
      "ogg",
      "invalid",
      "Verify sound:",
    ),
    (
      GamedataVerificationType::Spawns,
      "spawns",
      "spawn",
      "invalid",
      "Verify spawn file:",
    ),
  ] {
    let root = build_absolute_generated_test_resource_path(&format!("gamedata_cancellation/{check}"));

    fs::create_dir_all(root.join("configs")).expect("configs directory");
    fs::create_dir_all(root.join(directory)).expect("asset directory");
    fs::write(root.join("configs/system.ltx"), "[system]\nversion = 1\n").expect("system config");

    for name in ["first", "second", "third"] {
      fs::write(root.join(directory).join(format!("{name}.{extension}")), contents).expect("test asset");
    }

    let project = GamedataProject::open(&GamedataProjectReadOptions {
      root,
      ..Default::default()
    })
    .expect("project opens");

    let job = JobHandle::inert();
    let sink = Arc::new(CancelAfterFirstItem {
      job: job.clone(),
      prefix,
      items: AtomicUsize::new(0),
    });
    let options = GamedataProjectVerifyOptions {
      job,
      checks: vec![check],
      output: OutputOptions::new(sink.clone(), OutputVerbosity::Verbose),
      ..Default::default()
    };

    // A single worker makes the exact stop boundary deterministic, including nested parallel iterators.
    let pool = ThreadPoolBuilder::new().num_threads(1).build().expect("test pool");

    let mut report = crate::GamedataVerificationReport::default();
    report.add_report(GamedataVerificationType::Coverage.run(&project, &options));
    // Run the check directly: a whole sweep buffers its output until the check finishes.
    report.add_report(pool.install(|| check.run(&project, &options)));
    let interrupted = report
      .get_checks()
      .iter()
      .find(|it| it.get_verification_type() == check)
      .expect("started check remains visible");

    assert_eq!(
      sink.items.load(Ordering::Relaxed),
      1,
      "{check} kept visiting assets after cancellation"
    );
    assert_eq!(report.get_outcome(), JobOutcome::Cancelled, "{check}");
    assert_eq!(
      interrupted.get_status(),
      GamedataVerificationStatus::Incomplete,
      "{check}"
    );
    assert!(
      interrupted.get_findings().is_empty(),
      "cancelled work is not a validation finding"
    );
    assert!(
      report
        .get_checks()
        .iter()
        .any(|it| it.get_verification_type() == GamedataVerificationType::Coverage)
    );
  }
}
