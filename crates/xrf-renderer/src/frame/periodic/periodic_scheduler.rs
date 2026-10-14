use std::collections::{HashMap, VecDeque};

use xrf_renderer_core::GraphTaggedTime;

use crate::frame::periodic::periodic_job::PeriodicJob;
use crate::frame::periodic::periodic_ticket::PeriodicTicket;

/// What a frame was granted: each job's passes, the units it drew and what a unit cost before any was learned.
type FrameGrants = Vec<(&'static [&'static str], f32, f32)>;

/// The work every view and scene draws once in so many frames, and what streams in, decided for all of them so the frame
/// carries about the same share of it every time: each offers its due jobs, the scheduler grants them, and each does
/// what it was granted. A frame decides in rounds sharing its budget: what streams in as the scenes take their updates,
/// then what is drawn once every view prepared. A job that must be done this frame is granted whatever it costs, and
/// spends the budget like any other, so what may wait fills only what it leaves; then the one waited longest past its
/// deadline, whatever the budget, one a frame; then the rest by how near their deadline they stand, while they fit.
/// A job's cost is its units at the milliseconds a unit last cost: as the timer measured its passes where they are
/// timed, else as the frames' GPU times show it, fitted over the jobs each frame was granted; its seed before either.
pub struct PeriodicScheduler {
  /// Milliseconds of GPU time a frame's work that may wait fills.
  budget: f32,
  /// Frames begun, which tags the timed frames.
  frame: u64,
  /// Who offers what is offered next.
  owner: u32,
  /// This frame's offers, by their owner, whether withdrawn and whether granted; those before `decided` were decided by
  /// an earlier round.
  offers: Vec<PeriodicOffer>,
  decided: usize,
  /// Whether this frame took a job past its deadline over the budget.
  is_forced: bool,
  /// Milliseconds a unit costs, by a job's first pass, as measured.
  rates: HashMap<&'static str, f32>,
  /// The units a job draws as a rule, by its first pass, which the fit to the frames' GPU times weighs it by.
  typical_units: HashMap<&'static str, f32>,
  /// Milliseconds the GPU spends on a frame apart from what it was granted, as the fit finds it.
  baseline: Option<f32>,
  /// What each recent frame was granted, by the jobs' passes and units, for the timer to measure.
  granted: VecDeque<(u64, FrameGrants)>,
  /// Milliseconds this frame's grants of what may wait cost, and of what must draw.
  spent: f32,
  now_spent: f32,
}

struct PeriodicOffer {
  owner: u32,
  job: PeriodicJob,
  is_withdrawn: bool,
  is_granted: bool,
}

impl PeriodicScheduler {
  /// Milliseconds of GPU time a frame gives what may wait, unless the settings say otherwise.
  pub const DEFAULT_BUDGET: f32 = 0.5;
  /// Frames whose grants are kept for the timer, which reads a frame back a few frames late.
  const KEPT_FRAMES: usize = 8;
  /// How far one measurement moves a learned cost.
  const LEARNING: f32 = 0.25;
  /// How far one frame's GPU time moves the fit's costs and baseline: a normalised least-mean-squares step.
  const FIT_STEP: f32 = 0.2;
  /// How far one frame moves a job's typical units.
  const UNITS_STEP: f32 = 0.1;

  pub fn new() -> Self {
    Self {
      budget: Self::DEFAULT_BUDGET,
      frame: 0,
      owner: 0,
      offers: Vec::new(),
      decided: 0,
      is_forced: false,
      rates: HashMap::new(),
      typical_units: HashMap::new(),
      baseline: None,
      granted: VecDeque::new(),
      spent: 0.0,
      now_spent: 0.0,
    }
  }

  /// Starts a frame's offers with `budget` milliseconds for the work drawn once in so many frames, learning first from
  /// the passes the timer read back since the last frame, and from the GPU's milliseconds on each frame read back
  /// (`spans`, by the frame's tag) where its passes were not timed.
  pub fn begin(&mut self, budget: f32, (times, spans): (&[GraphTaggedTime], &[(u64, f32)])) {
    self.learn(times);
    self.fit(times, spans);
    self.budget = budget.max(0.0);
    self.frame += 1;
    self.owner = 0;
    self.offers.clear();
    self.decided = 0;
    self.is_forced = false;
    self.spent = 0.0;
    self.now_spent = 0.0;
  }

  /// The frame begun last, which tags it for the timer.
  pub fn get_frame(&self) -> u64 {
    self.frame
  }

  /// Milliseconds this frame's grants cost, as estimated: of what may wait, which the budget bounds, and of what must
  /// draw this frame.
  pub fn get_spent(&self) -> (f32, f32) {
    (self.spent, self.now_spent)
  }

  /// Names who offers the jobs offered from now on, which `withdraw` takes back together.
  pub fn set_owner(&mut self, owner: u32) {
    self.owner = owner;
  }

  /// Offers a job for this frame, granted or not once `grant` decides.
  pub fn offer(&mut self, job: PeriodicJob) -> PeriodicTicket {
    self.offers.push(PeriodicOffer {
      owner: self.owner,
      job,
      is_withdrawn: false,
      is_granted: false,
    });

    PeriodicTicket {
      frame: self.frame,
      index: self.offers.len() - 1,
    }
  }

  /// Takes back what `owner` offered this frame: a view that does not draw this frame draws none of it.
  pub fn withdraw(&mut self, owner: u32) {
    for offer in self.offers.iter_mut().filter(|offer| offer.owner == owner) {
      offer.is_withdrawn = true;
      offer.is_granted = false;
    }
  }

  /// Decides the offers since the last round: every one that must be done, then the one most overdue unless the frame
  /// took one already, then the rest most urgent first while they fit what is left of the budget.
  pub fn grant(&mut self) {
    let costs: Vec<f32> = self.offers.iter().map(|offer| self.get_cost(&offer.job)).collect();
    let open: Vec<usize> = (self.decided..self.offers.len())
      .filter(|index| !self.offers[*index].is_withdrawn)
      .collect();

    // What must draw this frame draws whatever it costs; it is steady work, not what the budget evens out.
    let now: Vec<usize> = open
      .iter()
      .copied()
      .filter(|index| self.offers[*index].job.is_now())
      .collect();

    for index in now {
      self.offers[index].is_granted = true;
      self.now_spent += costs[index];
    }

    let overdue: Option<usize> = open
      .iter()
      .copied()
      .filter(|index| !self.is_forced && self.offers[*index].job.is_overdue())
      .fold(None, |most: Option<usize>, index| {
        let past = |index: usize| {
          let job: &PeriodicJob = &self.offers[index].job;

          job.waited - job.wait_most
        };

        match most {
          Some(most) if past(most) >= past(index) => Some(most),
          _ => Some(index),
        }
      });

    if let Some(index) = overdue {
      self.offers[index].is_granted = true;
      self.spent += costs[index];
      self.is_forced = true;
    }

    let mut waiting: Vec<usize> = open
      .into_iter()
      .filter(|index| !self.offers[*index].is_granted)
      .collect();

    waiting.sort_by(|a, b| {
      self.offers[*b]
        .job
        .get_urgency()
        .total_cmp(&self.offers[*a].job.get_urgency())
    });

    for index in waiting {
      if self.spent + self.now_spent + costs[index] <= self.budget {
        self.offers[index].is_granted = true;
        self.spent += costs[index];
      }
    }

    let granted: FrameGrants = self.offers[self.decided..]
      .iter()
      .filter(|offer| offer.is_granted)
      .map(|offer| (offer.job.passes, offer.job.units, offer.job.seed))
      .collect();

    self.decided = self.offers.len();

    match self.granted.back_mut() {
      Some((frame, grants)) if *frame == self.frame => grants.extend(granted),
      _ => self.granted.push_back((self.frame, granted)),
    }

    while self.granted.len() > Self::KEPT_FRAMES {
      self.granted.pop_front();
    }
  }

  /// Whether a job offered this frame was granted.
  pub fn is_granted(&self, ticket: PeriodicTicket) -> bool {
    ticket.frame == self.frame && self.offers.get(ticket.index).is_some_and(|offer| offer.is_granted)
  }

  /// Milliseconds a job costs: its units at what a unit of its passes last cost, or at its seed before any was measured.
  pub fn get_cost(&self, job: &PeriodicJob) -> f32 {
    let rate: f32 = job
      .passes
      .first()
      .and_then(|pass| self.rates.get(pass))
      .copied()
      .unwrap_or(job.seed);

    job.units * rate
  }

  /// Moves each job's cost towards what its passes cost a unit in each frame read back: a pass the graph merged with
  /// others counts where every one merged is the job's own.
  fn learn(&mut self, times: &[GraphTaggedTime]) {
    for (frame, granted) in &self.granted {
      let frame_times: Vec<&GraphTaggedTime> = times.iter().filter(|time| time.tag == *frame).collect();

      if frame_times.is_empty() {
        continue;
      }

      let mut jobs: Vec<(&'static [&'static str], f32)> = Vec::new();

      for (passes, units, _) in granted {
        match jobs.iter_mut().find(|(seen, _)| seen.first() == passes.first()) {
          Some((_, total)) => *total += units,
          None => jobs.push((passes, *units)),
        }
      }

      for (passes, units) in jobs {
        let Some(key) = passes.first() else {
          continue;
        };
        let measured: Vec<f32> = frame_times
          .iter()
          .filter(|time| time.name.split(" + ").all(|part| passes.contains(&part)))
          .map(|time| time.gpu_time)
          .collect();

        if measured.is_empty() || units <= 0.0 {
          continue;
        }

        let rate: f32 = measured.iter().sum::<f32>() / units;
        let learned: &mut f32 = self.rates.entry(key).or_insert(rate);

        *learned += (rate - *learned) * Self::LEARNING;
      }
    }
  }
}

impl PeriodicScheduler {
  /// Fits each granted job's cost to the GPU's milliseconds on the frames read back whose passes were not timed: a
  /// frame's time is its baseline and its jobs' costs, each job weighed by its units against its typical ones, and each
  /// frame's error moves the baseline and the costs of the jobs it was granted by a normalised least-mean-squares step.
  /// Jobs granted together share the error until the scheduler grants them apart.
  fn fit(&mut self, times: &[GraphTaggedTime], spans: &[(u64, f32)]) {
    for &(frame, span) in spans {
      if times.iter().any(|time| time.tag == frame) {
        continue;
      }

      let Some((_, granted)) = self.granted.iter().find(|(tag, _)| *tag == frame) else {
        continue;
      };
      let mut jobs: Vec<(&'static str, f32, f32)> = Vec::new();

      for &(passes, units, seed) in granted {
        let Some(key) = passes.first().copied() else {
          continue;
        };

        match jobs.iter_mut().find(|(seen, ..)| *seen == key) {
          Some((_, total, _)) => *total += units,
          None => jobs.push((key, units, seed)),
        }
      }

      // Each job's units against its typical ones, and its typical draw's cost.
      let weighed: Vec<(&'static str, f32, f32, f32)> = jobs
        .iter()
        .filter(|(_, units, _)| *units > 0.0)
        .map(|&(key, units, seed)| {
          let typical: f32 = *self
            .typical_units
            .entry(key)
            .and_modify(|typical| *typical += (units - *typical) * Self::UNITS_STEP)
            .or_insert(units);
          let rate: f32 = self.rates.get(key).copied().unwrap_or(seed);

          (key, units / typical, rate * typical, typical)
        })
        .collect();
      let predicted: f32 = weighed.iter().map(|(_, share, draw, _)| share * draw).sum();
      let baseline: f32 = *self.baseline.get_or_insert(span - predicted);
      let error: f32 = span - baseline - predicted;
      let norm: f32 = 1.0 + weighed.iter().map(|(_, share, ..)| share * share).sum::<f32>();

      self.baseline = Some(baseline + Self::FIT_STEP * error / norm);

      for (key, share, draw, typical) in weighed {
        let fitted: f32 = (draw + Self::FIT_STEP * error * share / norm).max(0.0);

        self.rates.insert(key, fitted / typical);
      }
    }
  }
}

impl Default for PeriodicScheduler {
  fn default() -> Self {
    Self::new()
  }
}
