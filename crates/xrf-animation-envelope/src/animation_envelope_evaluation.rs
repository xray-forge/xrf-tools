//! `evalEnvelope` (`xrCore/Animation/interp.cpp`): the value of a LightWave envelope at a time, between its keys by
//! each key's shape and outside them by the envelope's behaviours.

use crate::{AnimationBehavior, AnimationEnvelope, AnimationInterpolation, AnimationKey, AnimationShape};

/// How close `bez2_time` brings the curve's time to the one asked for.
const BEZ2_TIME_TOLERANCE: f32 = 0.0001;

/// Halvings `bez2_time` takes at most: it recurses until within its tolerance, which a float reaches long before.
const BEZ2_TIME_STEPS: u32 = 64;

impl AnimationEnvelope {
  /// The channel's value at a time in seconds, `CEnvelope::Evaluate`: zero without keys, the one key's value with one,
  /// else the keys' curve at the time.
  pub fn evaluate(&self, time: f32) -> f32 {
    let keys: &[AnimationKey] = &self.keys;

    let (start, end) = match keys {
      [] => return 0.0,
      [only] => return only.value,
      [start, .., end] => (start, end),
    };
    let mut time: f32 = time;
    let mut offset: f32 = 0.0;

    if time < start.time || time > end.time {
      let (behavior, is_before): (AnimationBehavior, bool) = if time < start.time {
        (self.behavior_before, true)
      } else {
        (self.behavior_after, false)
      };

      match behavior {
        AnimationBehavior::Reset => return 0.0,
        AnimationBehavior::Constant => return if is_before { start.value } else { end.value },
        AnimationBehavior::Repeat => time = Self::to_range(time, start.time, end.time).0,
        AnimationBehavior::Oscillate => {
          let (ranged, wavelengths) = Self::to_range(time, start.time, end.time);

          // As the engine writes it: the span's length less the time, not its end.
          time = if wavelengths % 2 != 0 {
            end.time - start.time - ranged
          } else {
            ranged
          };
        }
        AnimationBehavior::Offset => {
          let (ranged, wavelengths) = Self::to_range(time, start.time, end.time);

          time = ranged;
          offset = wavelengths as f32 * (end.value - start.value);
        }
        AnimationBehavior::Linear if is_before => {
          let next: &AnimationKey = &keys[1];
          let slope: f32 = Self::get_outgoing(None, start, next) / (next.time - start.time);

          return slope * (time - start.time) + start.value;
        }
        AnimationBehavior::Linear => {
          let previous: &AnimationKey = &keys[keys.len() - 2];
          let slope: f32 = Self::get_incoming(previous, end, None) / (end.time - previous.time);

          return slope * (time - end.time) + end.value;
        }
        AnimationBehavior::Unknown(_) => {}
      }
    }

    // The span the time falls in, its keys and their neighbours. The search stops at the last span, where the engine
    // reads past its keys for a time an unnamed behaviour left beyond them.
    let mut index: usize = 0;

    while index + 2 < keys.len() && time > keys[index + 1].time {
      index += 1;
    }

    let (key0, key1): (&AnimationKey, &AnimationKey) = (&keys[index], &keys[index + 1]);
    let previous: Option<&AnimationKey> = index.checked_sub(1).map(|it| &keys[it]);
    let next: Option<&AnimationKey> = keys.get(index + 2);

    if time == key0.time {
      return key0.value + offset;
    }

    if time == key1.time {
      return key1.value + offset;
    }

    let t: f32 = (time - key0.time) / (key1.time - key0.time);

    match key1.shape {
      AnimationShape::Tcb | AnimationShape::Bezier | AnimationShape::Hermite => {
        let out: f32 = Self::get_outgoing(previous, key0, key1);
        let incoming: f32 = Self::get_incoming(key0, key1, next);
        let [h1, h2, h3, h4] = Self::get_hermite(t);

        h1 * key0.value + h2 * key1.value + h3 * out + h4 * incoming + offset
      }
      AnimationShape::TimedBezier => Self::get_bez2(key0, key1, time) + offset,
      AnimationShape::Linear => key0.value + t * (key1.value - key0.value) + offset,
      AnimationShape::Stepped => key0.value + offset,
      AnimationShape::Unknown(_) => offset,
    }
  }

  /// `range`: the time brought into `[low, high]`, and how many spans it was moved by.
  fn to_range(value: f32, low: f32, high: f32) -> (f32, i32) {
    let span: f32 = high - low;

    if span == 0.0 {
      return (low, 0);
    }

    let ranged: f32 = low + value - span * (value / span).floor();
    let rounding: f32 = if ranged > value { 0.5 } else { -0.5 };

    (ranged, -(((ranged - value) / span + rounding) as i32))
  }

  /// `hermite`: the four Hermite coefficients at `t`.
  fn get_hermite(t: f32) -> [f32; 4] {
    let t2: f32 = t * t;
    let t3: f32 = t * t2;
    let h2: f32 = 3.0 * t2 - t3 - t3;
    let h4: f32 = t3 - t2;

    [1.0 - h2, h2, h4 - t2 + t, h4]
  }

  /// `bezier`: a one-dimensional cubic Bezier curve at `t`.
  fn get_bezier(x0: f32, x1: f32, x2: f32, x3: f32, t: f32) -> f32 {
    let t2: f32 = t * t;
    let t3: f32 = t2 * t;
    let c: f32 = 3.0 * (x1 - x0);
    let b: f32 = 3.0 * (x2 - x1) - c;
    let a: f32 = x3 - x0 - c - b;

    a * t3 + b * t2 + c * t + x0
  }

  /// `bez2_time`: the curve parameter at which a `BEZ2` span's time curve reaches the time, by halving.
  fn get_bez2_parameter(x: [f32; 4], time: f32) -> f32 {
    let (mut t0, mut t1): (f32, f32) = (0.0, 1.0);
    let mut t: f32 = 0.5;

    for _ in 0..BEZ2_TIME_STEPS {
      t = t0 + (t1 - t0) * 0.5;

      let value: f32 = Self::get_bezier(x[0], x[1], x[2], x[3], t);

      if (time - value).abs() <= BEZ2_TIME_TOLERANCE {
        break;
      }

      if value > time {
        t1 = t;
      } else {
        t0 = t;
      }
    }

    t
  }

  /// `bez2`: a `BEZ2` span's value at a time.
  fn get_bez2(key0: &AnimationKey, key1: &AnimationKey, time: f32) -> f32 {
    let [_, p1, p2, p3] = key0.interpolation.parameters;
    let [q0, q1, ..] = key1.interpolation.parameters;
    let x: f32 = if key0.shape == AnimationShape::TimedBezier {
      key0.time + p2
    } else {
      key0.time + (key1.time - key0.time) / 3.0
    };
    let t: f32 = Self::get_bez2_parameter([key0.time, x, key1.time + q0, key1.time], time);
    let y: f32 = if key0.shape == AnimationShape::TimedBezier {
      key0.value + p3
    } else {
      key0.value + p1 / 3.0
    };
    Self::get_bezier(key0.value, y, q1 + key1.value, key1.value, t)
  }

  /// `outgoing`: the curve's tangent leaving `key0` towards `key1`.
  fn get_outgoing(previous: Option<&AnimationKey>, key0: &AnimationKey, key1: &AnimationKey) -> f32 {
    let AnimationInterpolation {
      tension,
      continuity,
      bias,
      parameters,
    } = key0.interpolation;
    let rise: f32 = key1.value - key0.value;
    let scale = |previous: &AnimationKey| (key1.time - key0.time) / (key1.time - previous.time);

    match key0.shape {
      AnimationShape::Tcb => {
        let a: f32 = (1.0 - tension) * (1.0 + continuity) * (1.0 + bias);
        let b: f32 = (1.0 - tension) * (1.0 - continuity) * (1.0 - bias);

        match previous {
          Some(previous) => scale(previous) * (a * (key0.value - previous.value) + b * rise),
          None => b * rise,
        }
      }
      AnimationShape::Linear => match previous {
        Some(previous) => scale(previous) * (key0.value - previous.value + rise),
        None => rise,
      },
      AnimationShape::Bezier | AnimationShape::Hermite => parameters[1] * previous.map_or(1.0, scale),
      AnimationShape::TimedBezier => {
        let out: f32 = parameters[3] * (key1.time - key0.time);

        if parameters[2].abs() > 1e-5 {
          out / parameters[2]
        } else {
          out * 1e5
        }
      }
      AnimationShape::Stepped | AnimationShape::Unknown(_) => 0.0,
    }
  }

  /// `incoming`: the curve's tangent arriving at `key1` from `key0`.
  fn get_incoming(key0: &AnimationKey, key1: &AnimationKey, next: Option<&AnimationKey>) -> f32 {
    let AnimationInterpolation {
      tension,
      continuity,
      bias,
      parameters,
    } = key1.interpolation;
    let rise: f32 = key1.value - key0.value;
    let scale = |next: &AnimationKey| (key1.time - key0.time) / (next.time - key0.time);

    match key1.shape {
      AnimationShape::Linear => match next {
        Some(next) => scale(next) * (next.value - key1.value + rise),
        None => rise,
      },
      AnimationShape::Tcb => {
        let a: f32 = (1.0 - tension) * (1.0 - continuity) * (1.0 + bias);
        let b: f32 = (1.0 - tension) * (1.0 + continuity) * (1.0 - bias);

        match next {
          Some(next) => scale(next) * (b * (next.value - key1.value) + a * rise),
          None => a * rise,
        }
      }
      AnimationShape::Bezier | AnimationShape::Hermite => parameters[0] * next.map_or(1.0, scale),
      AnimationShape::TimedBezier => {
        let incoming: f32 = parameters[1] * (key1.time - key0.time);

        if parameters[0].abs() > 1e-5 {
          incoming / parameters[0]
        } else {
          incoming * 1e5
        }
      }
      AnimationShape::Stepped | AnimationShape::Unknown(_) => 0.0,
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::{AnimationBehavior, AnimationEnvelope, AnimationInterpolation, AnimationKey, AnimationShape};

  fn key(time: f32, value: f32, shape: AnimationShape) -> AnimationKey {
    AnimationKey {
      value,
      time,
      shape,
      interpolation: AnimationInterpolation::default(),
    }
  }

  fn envelope(behavior: AnimationBehavior, keys: Vec<AnimationKey>) -> AnimationEnvelope {
    AnimationEnvelope {
      behavior_before: behavior,
      behavior_after: behavior,
      keys,
    }
  }

  fn assert_near(actual: f32, expected: f32) {
    assert!((actual - expected).abs() < 1e-5, "expected {expected}, got {actual}");
  }

  #[test]
  fn reads_zero_without_keys_and_the_one_key_alone() {
    assert_eq!(AnimationEnvelope::default().evaluate(3.0), 0.0);
    assert_eq!(
      envelope(AnimationBehavior::Constant, vec![key(1.0, 7.0, AnimationShape::Linear)]).evaluate(-5.0),
      7.0
    );
  }

  #[test]
  fn walks_a_line_and_holds_a_step() {
    let line: AnimationEnvelope = envelope(
      AnimationBehavior::Constant,
      vec![
        key(0.0, 2.0, AnimationShape::Linear),
        key(2.0, 6.0, AnimationShape::Linear),
      ],
    );
    let step: AnimationEnvelope = envelope(
      AnimationBehavior::Constant,
      vec![
        key(0.0, 2.0, AnimationShape::Stepped),
        key(2.0, 6.0, AnimationShape::Stepped),
      ],
    );

    assert_near(line.evaluate(0.5), 3.0);
    assert_near(step.evaluate(1.9), 2.0);
    assert_near(step.evaluate(2.0), 6.0);
  }

  // Two plain TCB keys: each tangent is the whole rise, so the Hermite curve is the straight line, h3 and h4 cancelling.
  #[test]
  fn draws_two_plain_tcb_keys_straight() {
    let tcb: AnimationEnvelope = envelope(
      AnimationBehavior::Constant,
      vec![key(0.0, 0.0, AnimationShape::Tcb), key(1.0, 4.0, AnimationShape::Tcb)],
    );

    assert_near(tcb.evaluate(0.25), 1.0);
    assert_near(tcb.evaluate(0.5), 2.0);
  }

  // Through a third key the middle tangent is (2 - 0 + 4 - 2) * 1 / 2 = 2 at t = 0.5 of the first span:
  // h1 = 0.5, h2 = 0.5, h3 = 0.125, h4 = -0.125; out of the first key 2 (b * rise), into the second 2.
  #[test]
  fn bends_tcb_through_its_neighbours() {
    let tcb: AnimationEnvelope = envelope(
      AnimationBehavior::Constant,
      vec![
        key(0.0, 0.0, AnimationShape::Tcb),
        key(1.0, 2.0, AnimationShape::Tcb),
        key(2.0, 4.0, AnimationShape::Tcb),
      ],
    );

    assert_near(tcb.evaluate(0.5), 1.0);
  }

  // A Hermite span takes the keys' own tangents: leaving the first at 2, arriving flat, h3 = 0.125 at t = 0.5.
  #[test]
  fn follows_hermite_tangents() {
    let mut leaving: AnimationKey = key(0.0, 0.0, AnimationShape::Hermite);
    let arriving: AnimationKey = key(1.0, 0.0, AnimationShape::Hermite);

    leaving.interpolation.parameters = [0.0, 2.0, 0.0, 0.0];

    assert_near(
      envelope(AnimationBehavior::Constant, vec![leaving, arriving]).evaluate(0.5),
      0.25,
    );
  }

  // A shape the engine does not name falls to its `default`, the offset alone. A behaviour it does not name leaves the
  // time as it is, which past the last key the engine reads beyond its keys for; here the last span is carried on.
  #[test]
  fn reads_unknown_codes_without_leaving_the_keys() {
    let shape: AnimationEnvelope = envelope(
      AnimationBehavior::Constant,
      vec![
        key(1.0, 10.0, AnimationShape::Linear),
        key(3.0, 20.0, AnimationShape::Unknown(7)),
      ],
    );
    let behavior: AnimationEnvelope = envelope(
      AnimationBehavior::Unknown(9),
      vec![
        key(1.0, 10.0, AnimationShape::Linear),
        key(3.0, 20.0, AnimationShape::Linear),
      ],
    );

    assert_eq!(shape.evaluate(2.0), 0.0);
    assert_eq!(shape.evaluate(1.0), 10.0);
    assert_near(behavior.evaluate(4.0), 25.0);
    assert_near(behavior.evaluate(0.0), 5.0);
  }

  #[test]
  fn behaves_outside_its_keys() {
    let keys = || {
      vec![
        key(1.0, 10.0, AnimationShape::Linear),
        key(3.0, 20.0, AnimationShape::Linear),
      ]
    };

    assert_eq!(envelope(AnimationBehavior::Reset, keys()).evaluate(0.0), 0.0);
    assert_eq!(envelope(AnimationBehavior::Constant, keys()).evaluate(9.0), 20.0);
    // `range` is `low + time - span * floor(time / span)`, not a shift by `low`: 4 s over a 2 s span from 1 reads 1 s.
    assert_near(envelope(AnimationBehavior::Repeat, keys()).evaluate(4.0), 10.0);
    // Offset: the same 1 s, two spans on by `range`'s count, so twice the rise of 10 added.
    assert_near(envelope(AnimationBehavior::Offset, keys()).evaluate(4.0), 30.0);
    // Linear: on along the slope of 5 a second.
    assert_near(envelope(AnimationBehavior::Linear, keys()).evaluate(5.0), 30.0);
    assert_near(envelope(AnimationBehavior::Linear, keys()).evaluate(0.0), 5.0);
  }

  // Oscillate as the engine writes it: 3.5 s ranges to 2.5 s one span on, an odd count, so it reads the span's length
  // less that, 2 - 2.5 = -0.5 s. That falls before the first key, and the line is extended back to it: t = -0.75.
  #[test]
  fn oscillates_as_the_engine_writes_it() {
    let keys: Vec<AnimationKey> = vec![
      key(1.0, 10.0, AnimationShape::Linear),
      key(3.0, 20.0, AnimationShape::Linear),
    ];

    assert_near(envelope(AnimationBehavior::Oscillate, keys).evaluate(3.5), 2.5);
  }
}
