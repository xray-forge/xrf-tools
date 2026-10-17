use xrf_renderer_core::GraphTaggedTime;

use crate::contract::render_viewport_id::RenderViewportId;
use crate::frame::periodic::periodic_job::PeriodicJob;
use crate::frame::periodic::periodic_owner::PeriodicOwner;
use crate::frame::periodic::periodic_scheduler::PeriodicScheduler;
use crate::frame::periodic::periodic_ticket::PeriodicTicket;

static PASSES: [&str; 2] = ["job cull", "job draw"];
static OTHER: [&str; 1] = ["other draw"];

fn waiting(cost: f32, (waited, wait_most): (u32, u32)) -> PeriodicJob {
  PeriodicJob::new_waiting(&PASSES, cost, 1.0, (waited, wait_most))
}

#[test]
fn grants_what_must_draw_whatever_the_budget() {
  let mut scheduler: PeriodicScheduler = PeriodicScheduler::new();

  scheduler.begin(0.5, (&[], &[]));

  let tickets: Vec<PeriodicTicket> = (0..3)
    .map(|_| scheduler.offer(PeriodicJob::new_now(&PASSES, 1.0, 1.0)))
    .collect();

  scheduler.grant();

  assert!(tickets.iter().all(|ticket| scheduler.is_granted(*ticket)));
  assert_eq!(scheduler.get_spent(), (0.0, 3.0));
}

#[test]
fn grants_the_most_urgent_within_the_budget() {
  let mut scheduler: PeriodicScheduler = PeriodicScheduler::new();

  scheduler.begin(0.5, (&[], &[]));

  let relaxed: PeriodicTicket = scheduler.offer(waiting(0.3, (0, 8)));
  let urgent: PeriodicTicket = scheduler.offer(waiting(0.3, (2, 4)));
  let small: PeriodicTicket = scheduler.offer(waiting(0.1, (0, 8)));

  scheduler.grant();

  assert!(scheduler.is_granted(urgent));
  assert!(!scheduler.is_granted(relaxed));
  assert!(scheduler.is_granted(small));
}

#[test]
fn takes_one_overdue_job_past_the_budget() {
  let mut scheduler: PeriodicScheduler = PeriodicScheduler::new();

  scheduler.begin(0.0, (&[], &[]));

  let late: PeriodicTicket = scheduler.offer(waiting(1.0, (4, 4)));
  let later: PeriodicTicket = scheduler.offer(waiting(1.0, (9, 4)));
  let waiting_job: PeriodicTicket = scheduler.offer(waiting(1.0, (1, 4)));

  scheduler.grant();

  assert!(scheduler.is_granted(later));
  assert!(!scheduler.is_granted(late));
  assert!(!scheduler.is_granted(waiting_job));
}

#[test]
fn withdraws_what_a_view_offered() {
  let mut scheduler: PeriodicScheduler = PeriodicScheduler::new();

  scheduler.begin(1.0, (&[], &[]));

  let streamed: PeriodicTicket = scheduler.offer(PeriodicJob::new_now(&PASSES, 0.1, 1.0));

  scheduler.set_owner(PeriodicOwner::View(RenderViewportId(0)));

  let drawn: PeriodicTicket = scheduler.offer(PeriodicJob::new_now(&PASSES, 0.1, 1.0));

  scheduler.set_owner(PeriodicOwner::View(RenderViewportId(1)));

  let withdrawn: PeriodicTicket = scheduler.offer(PeriodicJob::new_now(&PASSES, 0.1, 1.0));

  scheduler.withdraw(PeriodicOwner::View(RenderViewportId(1)));
  scheduler.grant();

  // The scenes' own offers stand apart from every view's, whatever its number.
  assert!(scheduler.is_granted(streamed));
  assert!(scheduler.is_granted(drawn));
  assert!(!scheduler.is_granted(withdrawn));
}

#[test]
fn answers_only_this_frame_s_tickets() {
  let mut scheduler: PeriodicScheduler = PeriodicScheduler::new();

  scheduler.begin(1.0, (&[], &[]));

  let ticket: PeriodicTicket = scheduler.offer(PeriodicJob::new_now(&PASSES, 0.1, 1.0));

  scheduler.grant();
  assert!(scheduler.is_granted(ticket));

  scheduler.begin(1.0, (&[], &[]));
  scheduler.offer(PeriodicJob::new_now(&PASSES, 0.1, 1.0));
  scheduler.grant();
  assert!(!scheduler.is_granted(ticket));
}

#[test]
fn learns_a_job_s_cost_from_its_passes() {
  let mut scheduler: PeriodicScheduler = PeriodicScheduler::new();
  let job: PeriodicJob = PeriodicJob::new_now(&PASSES, 0.001, 100.0);

  scheduler.begin(1.0, (&[], &[]));

  let frame: u64 = scheduler.get_frame();

  scheduler.offer(job);
  scheduler.offer(PeriodicJob::new_now(&OTHER, 0.5, 1.0));
  scheduler.grant();
  assert!((scheduler.get_cost(&job) - 0.1).abs() < 1e-6);

  let time = |name: &str, gpu_time: f32| GraphTaggedTime {
    tag: frame,
    name: name.to_owned(),
    gpu_time,
  };

  // The job's own passes, merged or not, count; one merged with another job's does not.
  scheduler.begin(
    1.0,
    (
      &[
        time("job cull", 0.2),
        time("job draw", 0.6),
        time("job draw + other draw", 5.0),
      ],
      &[],
    ),
  );

  assert!((scheduler.get_cost(&job) - 0.8).abs() < 1e-6);
}

#[test]
fn shares_a_frame_s_budget_and_its_overdue_job_across_rounds() {
  let mut scheduler: PeriodicScheduler = PeriodicScheduler::new();

  scheduler.begin(0.5, (&[], &[]));

  let streamed: PeriodicTicket = scheduler.offer(waiting(0.3, (0, 8)));
  let late: PeriodicTicket = scheduler.offer(waiting(0.1, (4, 4)));

  scheduler.grant();

  let drawn: PeriodicTicket = scheduler.offer(waiting(0.2, (0, 8)));
  let later: PeriodicTicket = scheduler.offer(waiting(1.0, (9, 4)));
  let small: PeriodicTicket = scheduler.offer(waiting(0.1, (0, 8)));

  scheduler.grant();

  // The first round spent 0.4 of 0.5 and took the frame's one overdue job; the second fits only what is left.
  assert!(scheduler.is_granted(streamed));
  assert!(scheduler.is_granted(late));
  assert!(!scheduler.is_granted(drawn));
  assert!(!scheduler.is_granted(later));
  assert!(scheduler.is_granted(small));
}

#[test]
fn spends_the_budget_on_what_must_draw_first() {
  let mut scheduler: PeriodicScheduler = PeriodicScheduler::new();

  scheduler.begin(0.5, (&[], &[]));
  scheduler.offer(PeriodicJob::new_now(&OTHER, 0.3, 1.0));

  let large: PeriodicTicket = scheduler.offer(waiting(0.3, (0, 8)));
  let small: PeriodicTicket = scheduler.offer(waiting(0.1, (0, 8)));

  scheduler.grant();

  assert!(!scheduler.is_granted(large));
  assert!(scheduler.is_granted(small));
}

#[test]
fn fits_a_job_s_cost_to_the_frames_gpu_times() {
  let mut scheduler: PeriodicScheduler = PeriodicScheduler::new();
  // Seeded at a tenth of a millisecond, it costs half a millisecond over a 5 ms frame, every third frame.
  let job: PeriodicJob = waiting(0.1, (0, 2));
  let mut spans: Vec<(u64, f32)> = Vec::new();

  for frame in 0..600 {
    scheduler.begin(1.0, (&[], &spans));
    spans.clear();

    let tag: u64 = scheduler.get_frame();
    let is_drawn: bool = frame % 3 == 0;

    if is_drawn {
      scheduler.offer(job);
    }

    scheduler.grant();
    spans.push((tag, 5.0 + if is_drawn { 0.5 } else { 0.0 }));
  }

  assert!(
    (scheduler.get_cost(&job) - 0.5).abs() < 0.05,
    "{}",
    scheduler.get_cost(&job)
  );
}
