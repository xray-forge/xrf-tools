use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant};

use xrf_audio_scene::AudioBus;
use xrf_test_utils::TestSkip;

use crate::control::audio_play::AudioPlay;
use crate::control::audio_refusal::AudioRefusal;
use crate::control::engine::audio_engine::AudioEngine;
use crate::control::engine::audio_engine_parts::AudioEngineParts;
use crate::decode::decoded_sound::DecodedSound;
use crate::decode::sound_channels::SoundChannels;
use crate::mixer::clock::audio_clock_anchor::AudioClockAnchor;
use crate::mixer::meter::audio_mixer_meter::AudioMixerMeter;
use crate::mixer::tap::audio_tap::AudioTap;
use crate::mixer::tap::audio_tap_block::AudioTapBlock;
use crate::mixer::tap::audio_tap_end::AudioTapEnd;
use crate::output::device::audio_output::AudioOutput;
use crate::output::device::audio_output_choice::AudioOutputChoice;
use crate::output::device::audio_output_status::AudioOutputStatus;
use crate::output::device::stream::output_thread::OutputThread;
use crate::output::offline::offline_render::OfflineRender;
use crate::tests::fixtures::{RATE, engine, looped, once, sine};

// The most a click may land from its flash.
const PICTURE_FRAME: Duration = Duration::from_nanos(16_666_667);

fn read_all(tap: &mut AudioTap) -> Vec<AudioTapBlock> {
  std::iter::from_fn(|| tap.read().expect("the tap's stream runs")).collect()
}

// Taps come back through a ring; one past its room is refused rather than lost on the audio thread.
#[test]
fn refuses_a_tap_past_those_the_mixer_can_hand_back() {
  let AudioEngineParts { mut engine, mixer } = engine(4);
  let mut render: OfflineRender = OfflineRender::new(mixer, RATE);
  let mut buffer: Vec<f32> = vec![0.0; 480 * 2];
  let first: AudioTap = engine.open_tap(Duration::from_secs(1)).expect("the first tap opens");
  let second: AudioTap = engine.open_tap(Duration::from_secs(1)).expect("the second tap opens");

  assert_eq!(
    engine.open_tap(Duration::from_secs(1)).err(),
    Some(AudioRefusal::TapsBusy)
  );

  drop((first, second));
  render.render_into(&mut buffer);

  assert!(engine.open_tap(Duration::from_secs(1)).is_ok());
}

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

// As when a new device opens at another rate.
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

// Answers how far from its flash the last anchor places the click's first tapped frame.
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

// With no device the clock is anchored at the frames rendered and moves only past 5 ms of drift.
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

// On a device the clock is anchored at each buffer it takes, its latency counted.
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
