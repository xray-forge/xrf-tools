use std::time::Duration;

use crate::contract::render_frame_rate::RenderFrameRate;
use crate::frame::periodic::periodic_budget::PeriodicBudget;

const FLOOR: f32 = 0.5;

fn capped(limit: u32) -> RenderFrameRate {
  RenderFrameRate {
    limit: Some(limit),
    is_vsync: false,
  }
}

/// Runs a second of frames `interval` milliseconds apart, the GPU spending `gpu` on each; the last frame's budget.
fn settle(budget: &mut PeriodicBudget, frame_rate: RenderFrameRate, (interval, gpu): (f32, Option<f32>)) -> f32 {
  let frames: u32 = (1000.0 / interval) as u32;

  (0..frames)
    .map(|_| budget.update((Duration::from_secs_f32(interval / 1000.0), gpu), frame_rate, FLOOR))
    .last()
    .unwrap_or(FLOOR)
}

#[test]
fn gives_the_floor_alone_to_uncapped_frames() {
  let uncapped: RenderFrameRate = RenderFrameRate {
    limit: None,
    is_vsync: false,
  };

  assert_eq!(
    settle(&mut PeriodicBudget::default(), uncapped, (6.0, Some(6.0))),
    FLOOR
  );
}

#[test]
fn gives_the_floor_alone_while_the_gpu_time_is_unknown() {
  assert_eq!(settle(&mut PeriodicBudget::default(), capped(60), (16.7, None)), FLOOR);
}

#[test]
fn gives_a_capped_frame_its_headroom_up_to_the_ceiling() {
  // 16.7 ms at 60 a second, 12 on the GPU, 1.67 of margin: about 3 ms left.
  let budget: f32 = settle(&mut PeriodicBudget::default(), capped(60), (16.7, Some(12.0)));

  assert!((budget - 3.0).abs() < 0.3, "{budget}");
  assert_eq!(
    settle(&mut PeriodicBudget::default(), capped(30), (33.3, Some(8.0))),
    PeriodicBudget::CEILING
  );
}

#[test]
fn gives_the_floor_to_a_frame_the_gpu_fills() {
  assert_eq!(
    settle(&mut PeriodicBudget::default(), capped(144), (6.9, Some(6.5))),
    FLOOR
  );
}

#[test]
fn takes_the_vsync_interval_from_the_fastest_frames() {
  let vsync: RenderFrameRate = RenderFrameRate::default();
  let mut budget: PeriodicBudget = PeriodicBudget::default();

  settle(&mut budget, vsync, (16.7, Some(12.0)));

  // One frame missing its refresh neither makes the interval longer nor the budget larger.
  let after_miss: f32 = budget.update((Duration::from_secs_f32(0.0334), Some(12.0)), vsync, FLOOR);

  assert!((after_miss - 3.0).abs() < 0.3, "{after_miss}");
}

#[test]
fn holds_the_budget_through_small_changes() {
  let mut budget: PeriodicBudget = PeriodicBudget::default();
  let held: f32 = settle(&mut budget, capped(60), (16.7, Some(12.0)));

  assert_eq!(settle(&mut budget, capped(60), (16.7, Some(12.1))), held);
}
