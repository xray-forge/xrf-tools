use std::time::Duration;

use crate::contract::render_frame_rate::RenderFrameRate;

#[test]
fn draws_at_the_display_refresh_by_default() {
  let rate: RenderFrameRate = RenderFrameRate::default();

  assert!(rate.is_vsync);
  assert_eq!(rate.get_interval(), None);
}

#[test]
fn paces_a_cap_and_reads_a_cap_of_zero_as_none() {
  let capped = RenderFrameRate {
    limit: Some(50),
    is_vsync: false,
  };
  let zero = RenderFrameRate {
    limit: Some(0),
    is_vsync: true,
  };

  assert_eq!(capped.get_interval(), Some(Duration::from_millis(20)));
  assert_eq!(zero.get_interval(), None);
}
