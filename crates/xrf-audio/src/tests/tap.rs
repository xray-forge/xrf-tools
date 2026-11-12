//! The mix tapped before the master, and the clock that places the mixer's frames: what a recorder takes its sound
//! from and lines it up with the picture by.

use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use xrf_test_utils::TestSkip;

use crate::output::device::stream::output_thread::OutputThread;
use crate::tests::fixtures::{RATE, engine, looped, once, sine};
use crate::{
  AudioBus, AudioClockAnchor, AudioEngine, AudioEngineParts, AudioMixerMeter, AudioOutput, AudioOutputChoice,
  AudioOutputStatus, AudioPlay, AudioTap, AudioTapBlock, AudioTapEnd, DecodedSound, OfflineRender, SoundChannels,
};

/// A frame of the picture at 60 a second: the most a click may land from its flash.
const PICTURE_FRAME: Duration = Duration::from_nanos(16_666_667);

/// Every block `tap` holds now.
fn read_all(tap: &mut AudioTap) -> Vec<AudioTapBlock> {
  std::iter::from_fn(|| tap.read().expect("the tap's stream runs")).collect()
}

// Taps opened faster than the mixer lets the earlier ones go are refused past the ring they come back through, rather
// than lost on the audio thread; once it has let them go, a tap opens again.
#[test]
fn refuses_a_tap_past_those_the_mixer_can_hand_back() {
  let AudioEngineParts { mut engine, mixer } = engine(4);
  let mut render: OfflineRender = OfflineRender::new(mixer, RATE);
  let mut buffer: Vec<f32> = vec![0.0; 480 * 2];
  let first: AudioTap = engine.open_tap(Duration::from_secs(1)).expect("the first tap opens");
  let second: AudioTap = engine.open_tap(Duration::from_secs(1)).expect("the second tap opens");

  assert_eq!(
    engine.open_tap(Duration::from_secs(1)).err(),
    Some(crate::AudioRefusal::TapsBusy)
  );

  drop((first, second));
  render.render_into(&mut buffer);

  assert!(engine.open_tap(Duration::from_secs(1)).is_ok());
}

// The tap holds the mix as the master takes it: the output is the tapped blocks at the master's gain, frame for frame,
// each block where the mixer's clock has it.
#[test]
fn takes_the_mix_before_the_master() {
  let sound: Arc<DecodedSound> = Arc::new(sine(RATE, RATE as usize, 440.0, 0.5, SoundChannels::Mono));
  let AudioEngineParts { mut engine, mixer } = engine(4);
  let mut render: OfflineRender = OfflineRender::new(mixer, RATE);
  let mut tap: AudioTap = engine.open_tap(Duration::from_secs(1)).expect("the tap opens");
  let mut out: Vec<f32> = Vec::new();
  let mut buffer: Vec<f32> = vec![0.0; 480 * 2];

  engine
    .play(AudioPlay::new(looped(&sound), AudioBus::Ambience))
    .expect("a voice starts");
  engine.set_master_gain(0.25).expect("the master takes its gain");

  for _ in 0..20 {
    render.render_into(&mut buffer);
    out.extend_from_slice(&buffer);
  }

  let blocks: Vec<AudioTapBlock> = read_all(&mut tap);

  assert_eq!(tap.get_rate(), RATE);
  assert_eq!(tap.get_dropped(), 0);
  assert_eq!(blocks.first().map(|block| block.frame), Some(0));
  assert!(
    blocks
      .windows(2)
      .all(|pair| pair[1].frame == pair[0].frame + AudioTapBlock::FRAMES as u64)
  );

  // Past the master's ramp of 10 ms.
  for block in blocks.iter().filter(|block| block.frame >= 1024) {
    let start: usize = block.frame as usize * 2;

    for (index, sample) in block.samples.iter().enumerate() {
      let Some(played) = out.get(start + index) else {
        break;
      };

      assert!(
        (played - sample * 0.25).abs() < 1e-6,
        "frame {}: {played} played, {sample} tapped",
        block.frame
      );
    }
  }

  assert!(
    blocks
      .iter()
      .any(|block| block.samples.iter().any(|sample| sample.abs() > 0.1))
  );
}

// A tap with no room drops whole blocks and counts them, the blocks either side keeping their frames.
#[test]
fn drops_and_counts_the_blocks_a_full_tap_has_no_room_for() {
  let AudioEngineParts { mut engine, mixer } = engine(4);
  let mut render: OfflineRender = OfflineRender::new(mixer, RATE);
  // Two blocks of room.
  let mut tap: AudioTap = engine
    .open_tap(Duration::from_secs_f64(512.0 / f64::from(RATE)))
    .expect("the tap opens");

  render.render(256 * 5);

  let blocks: Vec<AudioTapBlock> = read_all(&mut tap);

  assert_eq!(blocks.iter().map(|block| block.frame).collect::<Vec<u64>>(), [0, 256]);
  assert_eq!(tap.get_dropped(), 3);

  render.render(256);

  assert_eq!(read_all(&mut tap).first().map(|block| block.frame), Some(256 * 5));
}

// The mixer going on at another rate, as a new device opens, ends the tap's stream once what came before is read.
#[test]
fn ends_the_taps_stream_when_the_rate_changes() {
  let AudioEngineParts { mut engine, mut mixer } = engine(4);
  let mut out: Vec<f32> = vec![0.0; 256 * 2];

  mixer.set_output_rate(RATE);

  let mut tap: AudioTap = engine.open_tap(Duration::from_secs(1)).expect("the tap opens");

  mixer.render(&mut out, 2);
  mixer.set_output_rate(44_100);
  mixer.render(&mut out, 2);

  assert_eq!(
    tap
      .read()
      .expect("the block before the change")
      .map(|block| block.frame),
    Some(0)
  );
  assert_eq!(tap.read(), Err(AudioTapEnd::RateChanged { from: RATE, to: 44_100 }));
}

// The clock's anchor turns a frame into the instant it plays and back.
#[test]
fn places_a_frame_at_the_instant_it_plays() {
  let at: Instant = Instant::now();
  let anchor: AudioClockAnchor = AudioClockAnchor {
    frame: 48_000,
    at,
    latency: Duration::from_millis(20),
  };

  assert_eq!(anchor.get_instant(48_000, RATE), at + Duration::from_millis(20));
  assert_eq!(anchor.get_instant(96_000, RATE), at + Duration::from_millis(1020));
  assert_eq!(anchor.get_frame(at + Duration::from_millis(520), RATE), 72_000);
  assert_eq!(
    anchor.get_frame(at + Duration::from_millis(20) - Duration::from_millis(500), RATE),
    24_000
  );
}

/// Where a click stamped to play at a flash lands on the clock, with the mixer rendered by `render` meanwhile: the
/// flash `ahead` of now, the click a frame on the mixer's clock its anchor places there; answers how far from the flash
/// the anchor last written places the click's first tapped frame.
fn place_click(engine: &mut AudioEngine, meter: &AudioMixerMeter, mut render: impl FnMut()) -> Duration {
  let click: Arc<DecodedSound> = Arc::new(DecodedSound::new(RATE, SoundChannels::Mono, vec![0.5; 480]));
  let deadline: Instant = Instant::now() + Duration::from_secs(5);

  while meter.get_anchor().is_none() && Instant::now() < deadline {
    render();
  }

  let mut tap: AudioTap = engine.open_tap(Duration::from_secs(2)).expect("the tap opens");
  let flash: Instant = Instant::now() + Duration::from_millis(150);
  let rate: u32 = meter.get_rate();
  let stamped: u64 = meter
    .get_anchor()
    .expect("the clock is anchored")
    .get_frame(flash, rate);

  engine
    .play(AudioPlay {
      at: Some(stamped),
      ..AudioPlay::new(once(&click), AudioBus::Previews)
    })
    .expect("the click starts");

  let mut heard: Option<u64> = None;

  while heard.is_none() && Instant::now() < deadline {
    render();

    for block in read_all(&mut tap) {
      heard = heard.or_else(|| {
        block
          .samples
          .as_chunks::<2>()
          .0
          .iter()
          .position(|[left, _]| *left != 0.0)
          .map(|index| block.frame + index as u64)
      });
    }
  }

  let heard: u64 = heard.expect("the click is heard");
  let placed: Instant = meter
    .get_anchor()
    .expect("the clock is anchored")
    .get_instant(heard, rate);

  assert_eq!(heard, stamped, "the click starts on the frame it was stamped to");
  engine.collect();

  placed.max(flash).duration_since(placed.min(flash))
}

// With no device the mixer renders at a device's pace, its clock anchored at the frames rendered and moved only past
// 5 ms of drift; a click stamped to a flash lands within a picture's frame of it on that clock.
#[test]
fn places_a_click_at_its_flash_with_no_device() {
  let AudioEngineParts { mut engine, mut mixer } = engine(4);
  let (_requests, received) = mpsc::channel();

  mixer.set_output_rate(RATE);

  let meter: AudioMixerMeter = mixer.get_meter().clone();
  let mut output_thread: OutputThread = OutputThread::new(
    mixer,
    AudioOutputChoice::SystemDefault,
    received,
    Arc::new(Mutex::new(AudioOutputStatus::default())),
  );
  let mut render = || {
    output_thread.render_silent();
    std::thread::sleep(Duration::from_millis(5));
  };

  render();

  let first: AudioClockAnchor = meter.get_anchor().expect("the clock is anchored");
  let offset: Duration = place_click(&mut engine, &meter, &mut render);

  assert_eq!(first.latency, Duration::ZERO);
  assert_eq!(
    meter.get_anchor(),
    Some(first),
    "the anchor held at the pace it renders at"
  );
  assert!(offset < PICTURE_FRAME, "the click lands {offset:?} from its flash");
}

// On the machine's device the clock is anchored at each buffer it takes, its latency counted; a click stamped to a
// flash lands within a picture's frame of it there too. A machine with no device skips it.
#[test]
fn places_a_click_at_its_flash_on_a_device() {
  let AudioEngineParts { mut engine, mixer } = engine(4);
  let meter: AudioMixerMeter = mixer.get_meter().clone();
  let output: AudioOutput = AudioOutput::start(mixer, AudioOutputChoice::SystemDefault).expect("the output starts");
  let deadline: Instant = Instant::now() + Duration::from_secs(5);

  while output.get_status().device_name.is_none() && output.get_status().failure.is_none() && Instant::now() < deadline
  {
    std::thread::sleep(Duration::from_millis(10));
  }

  if output.get_status().device_name.is_none() {
    TestSkip::report(format_args!("no audio device: {:?}", output.get_status().failure));
    return;
  }

  // Past the buffers a device fills as it starts.
  std::thread::sleep(Duration::from_millis(500));

  let offset: Duration = place_click(&mut engine, &meter, || std::thread::sleep(Duration::from_millis(5)));

  assert!(offset < PICTURE_FRAME, "the click lands {offset:?} from its flash");
  drop(output);
}
