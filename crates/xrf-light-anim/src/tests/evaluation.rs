use crate::light_anim_item::LightAnimItem;
use crate::light_anim_key::LightAnimKey;

/// An animation of the given rate and length keyed at the given frames.
fn animation(fps: f32, frame_count: u32, keys: &[(i32, u32)]) -> LightAnimItem {
  LightAnimItem {
    name: String::from("test"),
    fps,
    frame_count,
    keys: keys
      .iter()
      .map(|(frame, color)| LightAnimKey {
        frame: *frame,
        color: *color,
      })
      .collect(),
  }
}

#[test]
fn answers_a_key_with_its_own_colour() {
  let item: LightAnimItem = animation(15.0, 10, &[(0, 0x1122_3344), (10, 0xFFFF_FFFF)]);

  assert_eq!(item.interpolate_rgb(0), Some(0x1122_3344));
  assert_eq!(item.interpolate_rgb(10), Some(0xFFFF_FFFF));
}

#[test]
fn blends_between_two_keys_rounding_each_channel_down() {
  // Halfway from 0 to 255 is 127.5 in every channel, alpha included; `color_argb_f` floors it to 127.
  let item: LightAnimItem = animation(15.0, 10, &[(0, 0x0000_0000), (10, 0xFFFF_FFFF)]);

  assert_eq!(item.interpolate_rgb(5), Some(0x7F7F_7F7F));
  assert_eq!(item.interpolate_rgb(1), Some(0x1919_1919));
}

#[test]
fn holds_the_last_key_past_it() {
  let item: LightAnimItem = animation(15.0, 10, &[(0, 0), (4, 0x00AB_CDEF)]);

  assert_eq!(item.interpolate_rgb(7), Some(0x00AB_CDEF));
}

#[test]
fn blends_from_a_key_before_frame_zero() {
  // `svet_01`'s shape: a key at -1 is what frame 0 blends from where frame 0 itself is not keyed.
  let item: LightAnimItem = animation(30.0, 30, &[(-1, 0x0000_0000), (1, 0x00FF_FFFF)]);

  assert_eq!(item.interpolate_rgb(0), Some(0x007F_7F7F));
}

#[test]
fn answers_nothing_where_the_engine_would_assert_or_read_past_its_keys() {
  let item: LightAnimItem = animation(15.0, 10, &[(2, 0x00FF_FFFF)]);

  assert_eq!(item.interpolate_rgb(11), None, "past the frame count");
  assert_eq!(item.interpolate_rgb(1), None, "before every key");
  assert_eq!(animation(15.0, 10, &[]).interpolate_rgb(0), None, "no keys");
}

#[test]
fn interpolates_over_the_keys_as_the_engine_loads_them() {
  // Listed out of order and with frame 10 twice, the second colour winning as the engine's map keeps it.
  let item: LightAnimItem = animation(15.0, 10, &[(10, 0x0000_0001), (0, 0), (10, 0xFFFF_FFFF)]);

  assert_eq!(item.interpolate_rgb(10), Some(0xFFFF_FFFF));
  assert_eq!(item.interpolate_rgb(5), Some(0x7F7F_7F7F));
}

#[test]
fn exchanges_red_and_blue_for_the_bgr_reading() {
  let item: LightAnimItem = animation(15.0, 1, &[(0, 0x80_102030)]);

  assert_eq!(item.interpolate_bgr(0), Some(0x80_302010));
  assert_eq!(item.calculate_bgr(0.0), Some((0, 0x80_302010)));
}

#[test]
fn takes_a_time_to_the_frame_it_falls_on_and_loops() {
  // Two seconds at fifteen frames: 2.5 s wraps to 0.5 s, which is frame 7.5, floored.
  let item: LightAnimItem = animation(15.0, 30, &[(0, 0x0000_0000), (30, 0x0000_001E)]);

  assert_eq!(item.calculate_rgb(0.0), Some((0, 0)));
  assert_eq!(item.calculate_rgb(2.5).map(|(frame, _)| frame), Some(7));
  assert_eq!(item.calculate_rgb(0.5), item.calculate_rgb(2.5));
}

#[test]
fn takes_a_negative_time_to_a_negative_frame_as_fmodf_does() {
  let item: LightAnimItem = animation(15.0, 30, &[(-10, 0x0000_0000), (0, 0x0000_0000)]);

  assert_eq!(item.calculate_rgb(-0.5).map(|(frame, _)| frame), Some(-8));
}

#[test]
fn has_no_time_for_a_rate_that_does_not_advance() {
  let item: LightAnimItem = animation(0.0, 30, &[(0, 0)]);

  assert_eq!(item.calculate_rgb(1.0), None);
  assert_eq!(item.get_length_ms(), None);
}

#[test]
fn measures_its_length_in_whole_milliseconds() {
  assert_eq!(animation(15.0, 30, &[(0, 0)]).get_length_ms(), Some(2000));
  // A third of a second is 333.33 ms, which `Length_ms` floors.
  assert_eq!(animation(30.0, 10, &[(0, 0)]).get_length_ms(), Some(333));
}
