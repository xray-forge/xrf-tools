use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::{Sender, channel};

use image::RgbaImage;
use xrf_error::XrfError;
use xrf_renderer::{BurstEvent, BurstMarks, RenderBurst, RenderCapture};
use xrf_test_utils::GeneratedResources;

use crate::viewer::LevelViewerFolder;
use crate::viewer::burst::{BurstOutcome, BurstWriter};

const REFERENCE: RenderBurst = RenderBurst {
  at: 600.0,
  step: 0.25,
  lead_in: 0,
  frames: 1,
  samples: Some(2),
};

/// A burst's folder, made empty.
fn folder(name: &str) -> PathBuf {
  let folder: PathBuf = GeneratedResources::path(&format!("burst_writer/{name}"));

  let _ = fs::remove_dir_all(&folder);
  fs::create_dir_all(&folder).expect("the folder to be made");

  folder
}

/// A two by one sample of frame 0: a flat shade, its marks `marks` in both texels.
fn send_sample(events: &Sender<BurstEvent>, sample: Option<u32>, (shade, marks): (u8, u8)) {
  events
    .send(BurstEvent::Shot {
      index: 0,
      sample,
      time: 600.0,
      picture: RenderCapture {
        width: 2,
        height: 1,
        pixels: [shade, shade, shade, 255].repeat(2),
        frame: 0,
      },
      marks: BurstMarks {
        width: 2,
        height: 1,
        marks: vec![marks; 2],
      },
    })
    .expect("the writer to listen");
}

// A reference's frame is its samples' mean, its marks' image the share of samples each mark was held in.
#[test]
fn writes_a_references_frame_as_its_samples_mean_and_its_marks_shares() {
  let folder: PathBuf = folder("reference");
  let (events, receiver) = channel();

  send_sample(&events, Some(0), (100, BurstMarks::PLANT | BurstMarks::GRASS));
  send_sample(&events, Some(1), (201, BurstMarks::PLANT));
  events.send(BurstEvent::Finished).expect("the writer to listen");

  let outcome: BurstOutcome = BurstWriter::new(folder.clone(), &REFERENCE)
    .receive(&receiver)
    .expect("the burst to be written");
  let picture: RgbaImage = image::open(folder.join("reference-000.png"))
    .expect("the frame to be written")
    .into_rgba8();
  let marks: RgbaImage = image::open(folder.join("marks-000.png"))
    .expect("the marks to be written")
    .into_rgba8();

  assert_eq!(outcome.times, vec![600.0]);
  assert_eq!((outcome.width, outcome.height), (2, 1));
  assert_eq!(picture.get_pixel(1, 0).0, [151, 151, 151, 255]);
  assert_eq!(marks.get_pixel(0, 0).0, [255, 128, 0, 255]);
}

// A burst's frames come one sample each, written as they come.
#[test]
fn writes_a_bursts_frame_as_it_comes() {
  let folder: PathBuf = folder("frames");
  let (events, receiver) = channel();

  send_sample(&events, None, (40, BurstMarks::MOVED));
  events.send(BurstEvent::Finished).expect("the writer to listen");

  BurstWriter::new(
    folder.clone(),
    &RenderBurst {
      samples: None,
      ..REFERENCE
    },
  )
  .receive(&receiver)
  .expect("the burst to be written");

  assert!(folder.join("frame-000.png").exists());
  assert_eq!(
    image::open(folder.join("marks-000.png"))
      .expect("the marks to be written")
      .into_rgba8()
      .get_pixel(0, 0)
      .0,
    [0, 0, 255, 255]
  );
}

#[test]
fn refuses_a_sample_out_of_order_and_a_burst_that_stopped() {
  let (events, receiver) = channel();

  send_sample(&events, Some(1), (0, 0));

  assert!(
    BurstWriter::new(folder("out-of-order"), &REFERENCE)
      .receive(&receiver)
      .is_err()
  );

  let (events, receiver) = channel();

  events
    .send(BurstEvent::Stopped {
      error: XrfError::new_invalid_error("gone"),
    })
    .expect("the writer to listen");

  assert!(
    BurstWriter::new(folder("stopped"), &REFERENCE)
      .receive(&receiver)
      .is_err()
  );
}

#[test]
fn refuses_a_burst_name_that_is_taken_or_not_one_file_name() {
  let bursts: PathBuf = folder("names");

  fs::create_dir_all(bursts.join("taken")).expect("the taken burst to be made");

  assert_eq!(
    LevelViewerFolder::find_new_burst(&bursts, "zaton").expect("a free name"),
    bursts.join("zaton")
  );

  for name in ["taken", "", ".hidden", "nested/zaton", "zaton?"] {
    assert!(
      LevelViewerFolder::find_new_burst(&bursts, name).is_err(),
      "'{name}' must not name a new burst"
    );
  }
}
