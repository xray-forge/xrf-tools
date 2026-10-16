use crate::contract::render_overlay::RenderOverlay;
use crate::contract::render_overlay_segment::RenderOverlaySegment;
use crate::scene::level::level_overlays::LevelOverlays;
use crate::tests::test_context::create_test_context;

fn lines(segments: usize, is_depth_tested: bool) -> RenderOverlay {
  RenderOverlay::Lines {
    segments: vec![
      RenderOverlaySegment {
        ends: [[0.0; 3]; 2],
        colors: [[1.0; 3]; 2],
      };
      segments
    ],
    is_depth_tested,
  }
}

/// Every set's segments as two vertices each, and the sun apart from them.
#[test]
fn packs_segments_and_the_sun() {
  let Some(context) = create_test_context("to draw with") else {
    return;
  };
  let overlays: LevelOverlays = LevelOverlays::new(
    &context.device,
    &[
      lines(1, true),
      lines(2, false),
      RenderOverlay::Sun {
        color: [1.0, 0.9, 0.8],
        size: 24.0,
      },
    ],
    5,
  );

  assert_eq!(overlays.version, 5);
  assert_eq!(overlays.lines.as_ref().map(|(_, count)| *count), Some(6));
  assert!(overlays.sun.is_some());

  let empty: LevelOverlays = LevelOverlays::new(&context.device, &[lines(0, true)], 6);

  assert!(empty.lines.is_none());
  assert!(empty.sun.is_none());
}

#[test]
fn reads_overlays_as_the_viewers_send_them() {
  let overlays: Vec<RenderOverlay> = serde_json::from_str(
    r#"[{"kind":"lines","segments":[{"ends":[[0,0,0],[1,0,0]],"colors":[[1,0,0],[1,0,0]]}],"isDepthTested":false},{"kind":"points","positions":[[0,1,0]],"color":[1,1,1],"size":8,"isDepthTested":false},{"kind":"sun","color":[1,1,1],"size":24}]"#,
  )
  .unwrap();

  assert_eq!(
    overlays,
    vec![
      RenderOverlay::Lines {
        segments: vec![RenderOverlaySegment {
          ends: [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
          colors: [[1.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
        }],
        is_depth_tested: false,
      },
      RenderOverlay::Points {
        positions: vec![[0.0, 1.0, 0.0]],
        color: [1.0, 1.0, 1.0],
        size: 8.0,
        is_depth_tested: false,
      },
      RenderOverlay::Sun {
        color: [1.0, 1.0, 1.0],
        size: 24.0,
      },
    ]
  );
}

/// A segment without its other end, or a point short of a coordinate, is refused rather than dropped.
#[test]
fn refuses_a_partial_segment_or_point() {
  assert!(
    serde_json::from_str::<RenderOverlay>(
      r#"{"kind":"lines","segments":[{"ends":[[0,0,0]],"colors":[[1,0,0],[1,0,0]]}],"isDepthTested":false}"#
    )
    .is_err()
  );
  assert!(
    serde_json::from_str::<RenderOverlay>(
      r#"{"kind":"points","positions":[[0,1]],"color":[1,1,1],"size":8,"isDepthTested":false}"#
    )
    .is_err()
  );
}
