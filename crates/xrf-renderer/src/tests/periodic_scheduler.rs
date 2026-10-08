use xrf_renderer_core::GraphTaggedTime;

use crate::frame::periodic_job::PeriodicJob;
use crate::frame::periodic_scheduler::PeriodicScheduler;
use crate::frame::periodic_ticket::PeriodicTicket;

static PASSES: [&str; 2] = ["job cull", "job draw"];
static OTHER: [&str; 1] = ["other draw"];

fn waiting(cost: f32, (waited, wait_most): (u32, u32)) -> PeriodicJob {
  PeriodicJob::new_waiting(&PASSES, cost, 1.0, (waited, wait_most))
}

#[test]
fn grants_what_must_draw_whatever_the_budget() {
  let mut scheduler: PeriodicScheduler = PeriodicScheduler::new();

  scheduler.begin(0.5, &[]);

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

  scheduler.begin(0.5, &[]);

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

  scheduler.begin(0.0, &[]);

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

  scheduler.begin(1.0, &[]);
  scheduler.set_owner(1);

  let drawn: PeriodicTicket = scheduler.offer(PeriodicJob::new_now(&PASSES, 0.1, 1.0));

  scheduler.set_owner(2);

  let withdrawn: PeriodicTicket = scheduler.offer(PeriodicJob::new_now(&PASSES, 0.1, 1.0));

  scheduler.withdraw(2);
  scheduler.grant();

  assert!(scheduler.is_granted(drawn));
  assert!(!scheduler.is_granted(withdrawn));
}

#[test]
fn answers_only_this_frame_s_tickets() {
  let mut scheduler: PeriodicScheduler = PeriodicScheduler::new();

  scheduler.begin(1.0, &[]);

  let ticket: PeriodicTicket = scheduler.offer(PeriodicJob::new_now(&PASSES, 0.1, 1.0));

  scheduler.grant();
  assert!(scheduler.is_granted(ticket));

  scheduler.begin(1.0, &[]);
  scheduler.offer(PeriodicJob::new_now(&PASSES, 0.1, 1.0));
  scheduler.grant();
  assert!(!scheduler.is_granted(ticket));
}

#[test]
fn learns_a_job_s_cost_from_its_passes() {
  let mut scheduler: PeriodicScheduler = PeriodicScheduler::new();
  let job: PeriodicJob = PeriodicJob::new_now(&PASSES, 0.001, 100.0);

  scheduler.begin(1.0, &[]);

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
    &[
      time("job cull", 0.2),
      time("job draw", 0.6),
      time("job draw + other draw", 5.0),
    ],
  );

  assert!((scheduler.get_cost(&job) - 0.8).abs() < 1e-6);
}
