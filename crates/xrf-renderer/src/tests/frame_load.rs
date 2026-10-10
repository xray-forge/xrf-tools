use crate::scene::level::frame_load::FrameLoad;

#[test]
fn frame_load_gives_a_light_frame_to_one_waiting_job() {
  let mut load: FrameLoad = FrameLoad::new(false);

  assert!(load.take(0));
  assert!(!load.take(0));
  assert!(!load.take(FrameLoad::WAIT_MOST - 1));
}

#[test]
fn frame_load_keeps_a_heavy_frame_from_a_job_until_it_waited_its_most() {
  let mut load: FrameLoad = FrameLoad::new(true);

  assert!(!load.take(0));
  assert!(!load.take(FrameLoad::WAIT_MOST - 1));
  assert!(load.take(FrameLoad::WAIT_MOST));
  assert!(!load.take(FrameLoad::WAIT_MOST + 3));
}

#[test]
fn frame_load_lets_a_job_waited_its_most_share_a_light_frame_with_one_other() {
  let mut load: FrameLoad = FrameLoad::new(false);

  assert!(load.take(0));
  assert!(load.take(FrameLoad::WAIT_MOST));
  assert!(!load.take(FrameLoad::WAIT_MOST));
}
