use crate::scene::texture::decoded_texture::DecodedTexture;
use crate::scene::texture::sequence::decoded_sequence::DecodedSequence;
use crate::scene::texture::sequence::texture_sequence::TextureSequence;
use crate::scene::texture::sequence::texture_sequence_timing::TextureSequenceTiming;

fn frame(width: u32, levels: usize, fill: u8) -> DecodedTexture {
  DecodedTexture {
    format: wgpu::TextureFormat::Rgba8Unorm,
    width,
    height: width,
    layers: 1,
    is_volume: false,
    levels: (0..levels)
      .map(|level| vec![fill; ((width >> level).max(1) * (width >> level).max(1) * 4) as usize])
      .collect(),
    layout: String::from("A8R8G8B8"),
    is_expanded: true,
  }
}

#[test]
fn reads_a_vanilla_frame_list() {
  // `textures\artifact\artifact_anim_dist.seq`, cut to three frames, with the archive's CRLF endings.
  let sequence: TextureSequence = TextureSequence::parse(
    "21\r\nartifact\\artifact_anim_dist_01\r\nartifact\\artifact_anim_dist_02\r\nartifact\\artifact_anim_dist_03",
  )
  .unwrap();

  assert_eq!(
    sequence.timing,
    TextureSequenceTiming {
      frame_ms: 47,
      is_cycled: false
    }
  );
  assert_eq!(
    sequence.frames,
    [
      r"artifact\artifact_anim_dist_01",
      r"artifact\artifact_anim_dist_02",
      r"artifact\artifact_anim_dist_03"
    ]
  );
}

#[test]
fn reads_a_cycled_list_and_skips_blank_lines_and_trims_names() {
  let sequence: TextureSequence =
    TextureSequence::parse("Cycled\n 25 fps\n\n  ui\\ui_noise_00 \n\t\nui\\ui_noise_01\n").unwrap();

  assert_eq!(
    sequence.timing,
    TextureSequenceTiming {
      frame_ms: 40,
      is_cycled: true
    }
  );
  assert_eq!(sequence.frames, [r"ui\ui_noise_00", r"ui\ui_noise_01"]);
}

#[test]
fn refuses_a_rate_the_engine_divides_by_zero_with() {
  for text in ["", "0\nframe", "fast\nframe", "-5\nframe", "1001\nframe", "cycled\n"] {
    assert!(TextureSequence::parse(text).is_err(), "'{text}' is read");
  }
}

#[test]
fn loops_its_frames_by_whole_frame_times() {
  let timing: TextureSequenceTiming = TextureSequenceTiming {
    frame_ms: 47,
    is_cycled: false,
  };

  assert_eq!(timing.get_frame(0, 15), 0);
  assert_eq!(timing.get_frame(46, 15), 0);
  assert_eq!(timing.get_frame(47, 15), 1);
  assert_eq!(timing.get_frame(47 * 14, 15), 14);
  assert_eq!(timing.get_frame(47 * 15, 15), 0);
  assert_eq!(timing.get_frame(47 * 31, 15), 1);
}

#[test]
fn plays_a_cycled_sequence_back_and_forth_its_ends_shown_twice() {
  let timing: TextureSequenceTiming = TextureSequenceTiming {
    frame_ms: 10,
    is_cycled: true,
  };
  let shown: Vec<u32> = (0..8).map(|step| timing.get_frame(step * 10, 3)).collect();

  // `apply_seq`: id = frame % 2n, and past n, n - 1 - id % n.
  assert_eq!(shown, [0, 1, 2, 2, 1, 0, 0, 1]);
}

#[test]
fn stacks_frames_as_layers_leaving_out_one_unlike_the_first() {
  let timing: TextureSequenceTiming = TextureSequenceTiming {
    frame_ms: 50,
    is_cycled: false,
  };
  let sequence: DecodedSequence = DecodedSequence::stack(
    "anim",
    [
      (String::from("a"), frame(4, 3, 1)),
      (String::from("smaller"), frame(2, 2, 9)),
      (String::from("b"), frame(4, 3, 2)),
    ],
    timing,
  )
  .unwrap();

  assert_eq!(sequence.get_count(), 2);
  assert_eq!(sequence.timing, timing);
  // Each level holds the first frame's bytes, then the second's.
  assert_eq!(sequence.frames.levels[0].len(), 2 * 4 * 4 * 4);
  assert_eq!(sequence.frames.levels[2], [[1; 4], [2; 4]].concat());
}

#[test]
fn stacks_nothing_from_no_frames() {
  let timing: TextureSequenceTiming = TextureSequenceTiming {
    frame_ms: 50,
    is_cycled: false,
  };

  assert!(DecodedSequence::stack("anim", [], timing).is_none());
}
