//! A viewport's recording: a frame kept a sixtieth of a second apart, read back through a ring of four buffers the
//! recorder holds, a frame due with none free dropped and counted rather than waited for, each told with the time it
//! was presented.

use std::sync::mpsc::{Receiver, channel};
use std::time::{Duration, Instant};

use crate::capture::record::record_event::RecordEvent;
use crate::capture::record::record_frame::RecordFrame;
use crate::capture::record::record_job::RecordJob;
use crate::capture::record::record_shot::RecordShot;
use crate::context::gpu_context::GpuContext;
use crate::tests::support::test_context::create_test_context;

/// The recording's size, odd across so a row of its copy is padded.
const SIZE: (u32, u32) = (17, 8);

/// A frame presented at `SIZE`, each texel's blue its column, green its row.
struct Presented {
  texture: wgpu::Texture,
  pixels: Vec<u8>,
}

fn present(context: &GpuContext) -> Presented {
  let texture: wgpu::Texture = context.device.create_texture(&wgpu::TextureDescriptor {
    label: Some("recorded"),
    size: wgpu::Extent3d {
      width: SIZE.0,
      height: SIZE.1,
      depth_or_array_layers: 1,
    },
    mip_level_count: 1,
    sample_count: 1,
    dimension: wgpu::TextureDimension::D2,
    format: RecordShot::FORMAT,
    usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::COPY_DST,
    view_formats: &[],
  });
  let pixels: Vec<u8> = (0..SIZE.1)
    .flat_map(|y| (0..SIZE.0).flat_map(move |x| [x as u8, y as u8, 200, 255]))
    .collect();

  context.queue.write_texture(
    texture.as_image_copy(),
    &pixels,
    wgpu::TexelCopyBufferLayout {
      offset: 0,
      bytes_per_row: Some(SIZE.0 * 4),
      rows_per_image: Some(SIZE.1),
    },
    wgpu::Extent3d {
      width: SIZE.0,
      height: SIZE.1,
      depth_or_array_layers: 1,
    },
  );

  Presented { texture, pixels }
}

/// A frame at `now` as the render thread draws one: its shot taken where one is due and a buffer free, copied out of
/// `presented`, submitted, ended and presented a millisecond on, then what came back polled for, the GPU waited on as a
/// later frame would find it done. Answers whether a shot was taken.
fn draw(job: &mut RecordJob, context: &GpuContext, presented: &Presented, now: Instant) -> bool {
  let shot: Option<RecordShot> = job.take_shot(&context.device, now);
  let is_shot: bool = shot.is_some();

  if let Some(shot) = shot {
    let mut encoder: wgpu::CommandEncoder = context.device.create_command_encoder(&Default::default());

    shot.encode(&mut encoder, &presented.texture);
    context.queue.submit([encoder.finish()]);
    job.advance(shot);
    job.present(now + Duration::from_millis(1));
  }

  context
    .device
    .poll(wgpu::PollType::wait_indefinitely())
    .expect("the GPU to finish");
  job.poll();

  is_shot
}

/// Every frame `events` holds now.
fn take_frames(events: &Receiver<RecordEvent>) -> Vec<RecordFrame> {
  events
    .try_iter()
    .map(|event| match event {
      RecordEvent::Frame(frame) => frame,
      RecordEvent::Stopped { error } => panic!("the recording stopped: {error}"),
    })
    .collect()
}

// A frame is kept a sixtieth of a second on from the last, a frame a little early or late keeping its place; each is
// told with its place and the time it was presented, and read back blue first, its rows unpadded.
#[test]
fn keeps_a_frame_a_sixtieth_apart_told_with_its_present_time() {
  let Some(context) = create_test_context("to record frames") else {
    return;
  };
  let presented: Presented = present(&context);
  let (events, received) = channel();
  let mut job: RecordJob = RecordJob::new(SIZE, events);
  let start: Instant = Instant::now();
  let shots: Vec<bool> = [0.0, 5.0, 16.0, 20.0, 34.0, 41.0, 49.0]
    .into_iter()
    .map(|ms: f64| {
      draw(
        &mut job,
        &context,
        &presented,
        start + Duration::from_secs_f64(ms / 1000.0),
      )
    })
    .collect();
  let frames: Vec<RecordFrame> = take_frames(&received);

  assert_eq!(shots, [true, false, true, false, true, false, true]);
  assert_eq!(
    frames.iter().map(|frame| frame.index).collect::<Vec<u64>>(),
    [0, 1, 2, 3]
  );
  assert_eq!(frames[1].presented, start + Duration::from_millis(17));
  assert_eq!(frames[0].get_size(), SIZE);

  for frame in frames {
    assert_eq!(frame.read().expect("the frame to be read"), presented.pixels);
  }
}

// A recorder holding every buffer is a frame due with none free, dropped and counted, never waited for; a buffer
// let go takes the next frame due.
#[test]
fn drops_a_frame_rather_than_waits_while_the_recorder_holds_every_buffer() {
  let Some(context) = create_test_context("to record frames") else {
    return;
  };
  let presented: Presented = present(&context);
  let (events, received) = channel();
  let mut job: RecordJob = RecordJob::new(SIZE, events);
  let start: Instant = Instant::now();
  let at = |sixtieth: u32| start + RecordJob::EVERY * sixtieth;

  for sixtieth in 0..RecordJob::RING as u32 {
    assert!(draw(&mut job, &context, &presented, at(sixtieth)));
  }

  let mut held: Vec<RecordFrame> = take_frames(&received);
  let taking: Instant = Instant::now();

  assert!(!draw(&mut job, &context, &presented, at(4)));
  assert!(!draw(&mut job, &context, &presented, at(5)));
  assert!(taking.elapsed() < Duration::from_secs(1), "a frame due waited");
  assert_eq!((held.len(), job.get_dropped()), (RecordJob::RING, 2));

  // The recorder reads one; its buffer comes back with the next poll.
  held.remove(0).read().expect("the frame to be read");
  job.poll();

  assert!(draw(&mut job, &context, &presented, at(6)));
  assert_eq!(
    take_frames(&received)
      .iter()
      .map(|frame| (frame.index, frame.dropped))
      .collect::<Vec<(u64, u64)>>(),
    [(6, 2)]
  );

  // One dropped unread goes back too.
  drop(held.remove(0));
  job.poll();

  assert!(draw(&mut job, &context, &presented, at(7)));
}

// The recorder gone ends the recording at its next frame back.
#[test]
fn ends_once_its_recorder_goes() {
  let Some(context) = create_test_context("to record frames") else {
    return;
  };
  let presented: Presented = present(&context);
  let (events, received) = channel();
  let mut job: RecordJob = RecordJob::new(SIZE, events);

  drop(received);
  draw(&mut job, &context, &presented, Instant::now());

  assert!(job.is_ended());
}
