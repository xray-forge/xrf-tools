//! `evalEnvelope` (`xrCore/Animation/interp.cpp`): the value of a LightWave envelope at a time, between its keys by
//! each key's shape and outside them by the envelope's behaviours.

use crate::{AnimationEnvelope, AnimationKey};

/// `SHAPE_TCB`: a Kochanek-Bartels spline through the key.
const SHAPE_TCB: u8 = 0;
/// `SHAPE_HERM`: a Hermite spline with the key's own tangents.
const SHAPE_HERM: u8 = 1;
/// `SHAPE_BEZI`: a Bezier spline with the key's own tangents.
const SHAPE_BEZI: u8 = 2;
/// `SHAPE_LINE`: straight towards the key.
const SHAPE_LINE: u8 = 3;
/// `SHAPE_STEP`: held until the key.
const SHAPE_STEP: u8 = 4;
/// `SHAPE_BEZ2`: a Bezier spline whose handles are times and values.
const SHAPE_BEZ2: u8 = 5;

/// `BEH_RESET`: zero outside the keys.
const BEH_RESET: u8 = 0;
/// `BEH_CONSTANT`: the nearest end key's value.
const BEH_CONSTANT: u8 = 1;
/// `BEH_REPEAT`: the keyed span over again.
const BEH_REPEAT: u8 = 2;
/// `BEH_OSCILLATE`: the keyed span forwards and back.
const BEH_OSCILLATE: u8 = 3;
/// `BEH_OFFSET`: the keyed span over again, shifted by its rise each time.
const BEH_OFFSET: u8 = 4;
/// `BEH_LINEAR`: on along the end key's tangent.
const BEH_LINEAR: u8 = 5;

/// How close `bez2_time` brings the curve's time to the one asked for.
const BEZ2_TIME_TOLERANCE: f32 = 0.0001;

/// Halvings `bez2_time` takes at most: it recurses until within its tolerance, which a float reaches long before.
const BEZ2_TIME_STEPS: u32 = 64;

/// `evalEnvelope`: zero without keys, the one key's value with one, else the keys' curve at the time.
pub(crate) fn evaluate(envelope: &AnimationEnvelope, time: f32) -> f32 {
  let keys: &[AnimationKey] = &envelope.keys;

  let (start, end) = match keys {
    [] => return 0.0,
    [only] => return only.value,
    [start, .., end] => (start, end),
  };
  let mut time: f32 = time;
  let mut offset: f32 = 0.0;

  if time < start.time || time > end.time {
    let (behavior, is_before) = if time < start.time {
      (envelope.behavior.0, true)
    } else {
      (envelope.behavior.1, false)
    };

    match behavior {
      BEH_RESET => return 0.0,
      BEH_CONSTANT => return if is_before { start.value } else { end.value },
      BEH_REPEAT => time = to_range(time, start.time, end.time).0,
      BEH_OSCILLATE => {
        let (ranged, wavelengths) = to_range(time, start.time, end.time);

        // As the engine writes it: the span's length less the time, not its end.
        time = if wavelengths % 2 != 0 {
          end.time - start.time - ranged
        } else {
          ranged
        };
      }
      BEH_OFFSET => {
        let (ranged, wavelengths) = to_range(time, start.time, end.time);

        time = ranged;
        offset = wavelengths as f32 * (end.value - start.value);
      }
      BEH_LINEAR if is_before => {
        let next: &AnimationKey = &keys[1];
        let slope: f32 = get_outgoing(None, start, next) / (next.time - start.time);

        return slope * (time - start.time) + start.value;
      }
      BEH_LINEAR => {
        let previous: &AnimationKey = &keys[keys.len() - 2];
        let slope: f32 = get_incoming(previous, end, None) / (end.time - previous.time);

        return slope * (time - end.time) + end.value;
      }
      _ => {}
    }
  }

  // The span the time falls in, its keys and their neighbours.
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
    SHAPE_TCB | SHAPE_BEZI | SHAPE_HERM => {
      let out: f32 = get_outgoing(previous, key0, key1);
      let incoming: f32 = get_incoming(key0, key1, next);
      let [h1, h2, h3, h4] = get_hermite(t);

      h1 * key0.value + h2 * key1.value + h3 * out + h4 * incoming + offset
    }
    SHAPE_BEZ2 => get_bez2(key0, key1, time) + offset,
    SHAPE_LINE => key0.value + t * (key1.value - key0.value) + offset,
    SHAPE_STEP => key0.value + offset,
    _ => offset,
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

    let value: f32 = get_bezier(x[0], x[1], x[2], x[3], t);

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
  let [_, p1, p2, p3] = key0.get_parameters();
  let [q0, q1, ..] = key1.get_parameters();
  let x: f32 = if key0.shape == SHAPE_BEZ2 {
    key0.time + p2
  } else {
    key0.time + (key1.time - key0.time) / 3.0
  };
  let t: f32 = get_bez2_parameter([key0.time, x, key1.time + q0, key1.time], time);
  let y: f32 = if key0.shape == SHAPE_BEZ2 {
    key0.value + p3
  } else {
    key0.value + p1 / 3.0
  };
  get_bezier(key0.value, y, q1 + key1.value, key1.value, t)
}

/// `outgoing`: the curve's tangent leaving `key0` towards `key1`.
fn get_outgoing(previous: Option<&AnimationKey>, key0: &AnimationKey, key1: &AnimationKey) -> f32 {
  let [tension, continuity, bias] = key0.get_tcb();
  let parameters: [f32; 4] = key0.get_parameters();
  let rise: f32 = key1.value - key0.value;
  let scale = |previous: &AnimationKey| (key1.time - key0.time) / (key1.time - previous.time);

  match key0.shape {
    SHAPE_TCB => {
      let a: f32 = (1.0 - tension) * (1.0 + continuity) * (1.0 + bias);
      let b: f32 = (1.0 - tension) * (1.0 - continuity) * (1.0 - bias);

      match previous {
        Some(previous) => scale(previous) * (a * (key0.value - previous.value) + b * rise),
        None => b * rise,
      }
    }
    SHAPE_LINE => match previous {
      Some(previous) => scale(previous) * (key0.value - previous.value + rise),
      None => rise,
    },
    SHAPE_BEZI | SHAPE_HERM => parameters[1] * previous.map_or(1.0, scale),
    SHAPE_BEZ2 => {
      let out: f32 = parameters[3] * (key1.time - key0.time);

      if parameters[2].abs() > 1e-5 {
        out / parameters[2]
      } else {
        out * 1e5
      }
    }
    _ => 0.0,
  }
}

/// `incoming`: the curve's tangent arriving at `key1` from `key0`.
fn get_incoming(key0: &AnimationKey, key1: &AnimationKey, next: Option<&AnimationKey>) -> f32 {
  let [tension, continuity, bias] = key1.get_tcb();
  let parameters: [f32; 4] = key1.get_parameters();
  let rise: f32 = key1.value - key0.value;
  let scale = |next: &AnimationKey| (key1.time - key0.time) / (next.time - key0.time);

  match key1.shape {
    SHAPE_LINE => match next {
      Some(next) => scale(next) * (next.value - key1.value + rise),
      None => rise,
    },
    SHAPE_TCB => {
      let a: f32 = (1.0 - tension) * (1.0 - continuity) * (1.0 + bias);
      let b: f32 = (1.0 - tension) * (1.0 + continuity) * (1.0 - bias);

      match next {
        Some(next) => scale(next) * (b * (next.value - key1.value) + a * rise),
        None => a * rise,
      }
    }
    SHAPE_BEZI | SHAPE_HERM => parameters[0] * next.map_or(1.0, scale),
    SHAPE_BEZ2 => {
      let incoming: f32 = parameters[1] * (key1.time - key0.time);

      if parameters[0].abs() > 1e-5 {
        incoming / parameters[0]
      } else {
        incoming * 1e5
      }
    }
    _ => 0.0,
  }
}

#[cfg(test)]
mod tests {
  use crate::{AnimationEnvelope, AnimationInterpolation, AnimationKey};

  fn key(time: f32, value: f32, shape: u8) -> AnimationKey {
    AnimationKey {
      value,
      time,
      shape,
      interpolation: (shape != 4).then_some(AnimationInterpolation {
        tension: 0.0,
        continuity: 0.0,
        bias: 0.0,
        parameters: [0.0; 4],
      }),
    }
  }

  fn envelope(behavior: (u8, u8), keys: Vec<AnimationKey>) -> AnimationEnvelope {
    AnimationEnvelope { behavior, keys }
  }

  fn assert_near(actual: f32, expected: f32) {
    assert!((actual - expected).abs() < 1e-5, "expected {expected}, got {actual}");
  }

  #[test]
  fn reads_zero_without_keys_and_the_one_key_alone() {
    assert_eq!(envelope((1, 1), vec![]).evaluate(3.0), 0.0);
    assert_eq!(envelope((1, 1), vec![key(1.0, 7.0, 3)]).evaluate(-5.0), 7.0);
  }

  #[test]
  fn walks_a_line_and_holds_a_step() {
    let line: AnimationEnvelope = envelope((1, 1), vec![key(0.0, 2.0, 3), key(2.0, 6.0, 3)]);
    let step: AnimationEnvelope = envelope((1, 1), vec![key(0.0, 2.0, 4), key(2.0, 6.0, 4)]);

    assert_near(line.evaluate(0.5), 3.0);
    assert_near(step.evaluate(1.9), 2.0);
    assert_near(step.evaluate(2.0), 6.0);
  }

  // Two plain TCB keys: each tangent is the whole rise, so the Hermite curve is the straight line, h3 and h4 cancelling.
  #[test]
  fn draws_two_plain_tcb_keys_straight() {
    let tcb: AnimationEnvelope = envelope((1, 1), vec![key(0.0, 0.0, 0), key(1.0, 4.0, 0)]);

    assert_near(tcb.evaluate(0.25), 1.0);
    assert_near(tcb.evaluate(0.5), 2.0);
  }

  // Through a third key the middle tangent is (2 - 0 + 4 - 2) * 1 / 2 = 2 at t = 0.5 of the first span:
  // h1 = 0.5, h2 = 0.5, h3 = 0.125, h4 = -0.125; out of the first key 2 (b * rise), into the second 2.
  #[test]
  fn bends_tcb_through_its_neighbours() {
    let tcb: AnimationEnvelope = envelope((1, 1), vec![key(0.0, 0.0, 0), key(1.0, 2.0, 0), key(2.0, 4.0, 0)]);

    assert_near(tcb.evaluate(0.5), 1.0);
  }

  // A Hermite span takes the keys' own tangents: leaving the first at 2, arriving flat, h3 = 0.125 at t = 0.5.
  #[test]
  fn follows_hermite_tangents() {
    let mut leaving: AnimationKey = key(0.0, 0.0, 1);
    let arriving: AnimationKey = key(1.0, 0.0, 1);

    leaving.interpolation = leaving.interpolation.map(|it| AnimationInterpolation {
      parameters: [0.0, 2.0, 0.0, 0.0],
      ..it
    });

    assert_near(envelope((1, 1), vec![leaving, arriving]).evaluate(0.5), 0.25);
  }

  #[test]
  fn behaves_outside_its_keys() {
    let keys = || vec![key(1.0, 10.0, 3), key(3.0, 20.0, 3)];

    assert_eq!(envelope((0, 0), keys()).evaluate(0.0), 0.0);
    assert_eq!(envelope((1, 1), keys()).evaluate(9.0), 20.0);
    // `range` is `low + time - span * floor(time / span)`, not a shift by `low`: 4 s over a 2 s span from 1 reads 1 s.
    assert_near(envelope((2, 2), keys()).evaluate(4.0), 10.0);
    // Offset: the same 1 s, two spans on by `range`'s count, so twice the rise of 10 added.
    assert_near(envelope((4, 4), keys()).evaluate(4.0), 30.0);
    // Linear: on along the slope of 5 a second.
    assert_near(envelope((5, 5), keys()).evaluate(5.0), 30.0);
    assert_near(envelope((5, 5), keys()).evaluate(0.0), 5.0);
  }

  // Oscillate as the engine writes it: 3.5 s ranges to 2.5 s one span on, an odd count, so it reads the span's length
  // less that, 2 - 2.5 = -0.5 s. That falls before the first key, and the line is extended back to it: t = -0.75.
  #[test]
  fn oscillates_as_the_engine_writes_it() {
    let keys: Vec<AnimationKey> = vec![key(1.0, 10.0, 3), key(3.0, 20.0, 3)];

    assert_near(envelope((3, 3), keys).evaluate(3.5), 2.5);
  }
}
