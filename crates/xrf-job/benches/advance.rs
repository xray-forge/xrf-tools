//! What reporting costs the loop that reports.
//!
//! `advance` is called once per archive entry, per verified asset, per formatted config — by every worker, in the
//! hottest loop these operations have. The emission rule was chosen on the claim that reading the clock there is
//! negligible against the work being reported; this measures the claim rather than restating it.

use std::sync::Arc;
use std::time::Duration;

use criterion::{Criterion, criterion_group, criterion_main};
use xrf_job::{JobHandle, JobProgress, JobScope, NoopSink, ProgressSink};

/// A sink that counts, so an emission cannot be optimized away as doing nothing.
#[derive(Default)]
struct CountingSink {
  reported: std::sync::atomic::AtomicUsize,
}

impl ProgressSink for CountingSink {
  fn report(&self, _: &JobProgress) {
    self.reported.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
  }
}

fn advance(criterion: &mut Criterion) {
  let mut group = criterion.benchmark_group("advance");

  // What the command line and the library default pay: reporting is off, so the emission path is never entered.
  group.bench_function("inert", |bencher| {
    let job: JobHandle = JobHandle::inert();
    let writing: JobScope = job.enter("write", Some(u64::MAX));

    bencher.iter(|| writing.advance());
  });

  // What a watched run pays. The interval is the production default, so all but a handful of these read the clock,
  // find nothing due, and return - which is the case that has to be cheap.
  group.bench_function("watched", |bencher| {
    let job: JobHandle = JobHandle::with_interval(Arc::new(NoopSink), Duration::from_millis(100));
    let writing: JobScope = job.enter("write", Some(u64::MAX));

    bencher.iter(|| writing.advance());
  });

  // The emission itself, which builds a snapshot and hands it to a sink. Bounded to ten a second in production, so
  // this is the cost that does not scale with entry count - measured to keep it that way.
  group.bench_function("emitting", |bencher| {
    let job: JobHandle = JobHandle::with_interval(Arc::new(CountingSink::default()), Duration::ZERO);
    let writing: JobScope = job.enter("write", Some(u64::MAX));

    bencher.iter(|| writing.advance());
  });

  // What packing adds on top, because it names the entry it is on. Sequential work only: an allocation and a lock per
  // entry would be wasted on a parallel run whose current entry is meaningless anyway.
  group.bench_function("naming the entry", |bencher| {
    let job: JobHandle = JobHandle::with_interval(Arc::new(NoopSink), Duration::from_millis(100));
    let writing: JobScope = job.enter("write", Some(u64::MAX));

    bencher.iter(|| {
      job.set_detail(Some(String::from("meshes/actors/stalker/stalker_neutral_nauchniy.ogf")));
      writing.advance();
    });
  });

  group.finish();
}

criterion_group!(benches, advance);
criterion_main!(benches);
