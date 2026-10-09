use crate::contract::render_view_output::RenderViewOutput;

fn output(output_height: Option<u32>, output_scale: f32) -> RenderViewOutput {
  RenderViewOutput {
    output_height,
    output_scale,
    ..RenderViewOutput::default()
  }
}

#[test]
fn resolves_at_the_viewport_s_size_times_its_scale_from_half_to_twice() {
  assert_eq!(output(None, 1.0).get_output_size((2830, 1930)), (2830, 1930));
  assert_eq!(output(None, 0.5).get_output_size((2830, 1930)), (1415, 965));
  assert_eq!(output(None, 2.0).get_output_size((1280, 720)), (2560, 1440));
  assert_eq!(output(None, 0.1).get_output_size((1280, 720)), (640, 360));
  assert_eq!(output(None, 4.0).get_output_size((1280, 720)), (2560, 1440));
}

#[test]
fn resolves_a_height_asked_in_the_viewport_s_shape_never_past_twice_it() {
  assert_eq!(output(Some(1080), 1.0).get_output_size((2830, 1930)), (1584, 1080));
  assert_eq!(output(Some(720), 0.5).get_output_size((2830, 1930)), (1056, 720));
  assert_eq!(output(Some(2160), 1.0).get_output_size((1000, 500)), (2000, 1000));
}
