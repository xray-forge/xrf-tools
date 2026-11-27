use env_filter::Filter;
use log::{Level, Metadata};

use crate::log_filter::LogFilter;

fn passes(filter: &Filter, target: &str, level: Level) -> bool {
  filter.enabled(&Metadata::builder().target(target).level(level).build())
}

#[test]
fn holds_the_gpu_stack_to_warnings_by_default_and_the_rest_at_the_default_level() {
  let filter: Filter = LogFilter::without_directives().build_filter();

  for target in [
    "naga::proc::overloads::list",
    "wgpu_core::device",
    "wgpu_hal::dx12",
    "wgpu",
  ] {
    assert!(!passes(&filter, target, Level::Info), "{target} passes at info");
    assert!(passes(&filter, target, Level::Warn), "{target} is held at warn");
  }

  assert!(passes(&filter, "xrf_renderer::thread", Level::Info));
}

#[test]
fn takes_every_directive_as_written_over_the_default() {
  let filter: Filter = LogFilter::from_directives("naga=debug,xrf_world=warn").build_filter();

  assert!(passes(&filter, "naga::front", Level::Debug));
  assert!(!passes(&filter, "xrf_world::level", Level::Info));
}
