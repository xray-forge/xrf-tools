use crate::contract::render_scale::RenderScale;

#[test]
fn draws_each_side_over_the_ratio_truncated_as_the_upscalers_do() {
  assert_eq!(RenderScale::Native.get_drawn(2830), 2830);
  assert_eq!(RenderScale::Quality.get_drawn(2830), 1886);
  assert_eq!(RenderScale::Balanced.get_drawn(1930), 1135);
  assert_eq!(RenderScale::Performance.get_drawn(1415), 707);
  assert_eq!(
    (
      RenderScale::UltraPerformance.get_drawn(2830),
      RenderScale::UltraPerformance.get_drawn(1930)
    ),
    (943, 643)
  );
}

#[test]
fn never_draws_a_side_of_nothing() {
  assert_eq!(RenderScale::UltraPerformance.get_drawn(2), 1);
  assert_eq!(RenderScale::Performance.get_drawn(0), 1);
}
