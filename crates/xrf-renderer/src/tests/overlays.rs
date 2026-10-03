use crate::context::gpu_context::GpuContext;
use crate::context::render_backend::RenderBackend;
use crate::contract::render_overlay::RenderOverlay;
use crate::scene::level::level_overlays::LevelOverlays;

fn lines(vertices: usize, is_depth_tested: bool) -> RenderOverlay {
  RenderOverlay::Lines {
    positions: vec![0.0; vertices * 3],
    colors: vec![1.0; vertices * 3],
    is_depth_tested,
  }
}

/// Every set's segments whole, an odd end dropped with its own set, and the sun apart from them.
#[test]
fn packs_whole_segments_and_the_sun() {
  let context: GpuContext = match GpuContext::create_headless(RenderBackend::D3d12)
    .or_else(|_| GpuContext::create_headless(RenderBackend::Vulkan))
  {
    Ok(context) => context,
    Err(error) => {
      eprintln!("Skipped: no GPU to draw with ({error})");

      return;
    }
  };
  let overlays: LevelOverlays = LevelOverlays::new(
    &context.device,
    &[
      lines(3, true),
      lines(4, false),
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

  let empty: LevelOverlays = LevelOverlays::new(&context.device, &[lines(1, true)], 6);

  assert!(empty.lines.is_none());
  assert!(empty.sun.is_none());
}

#[test]
fn reads_overlays_as_the_viewers_send_them() {
  let overlays: Vec<RenderOverlay> = serde_json::from_str(
    r#"[{"kind":"lines","positions":[0,0,0,1,0,0],"colors":[1,0,0,1,0,0],"isDepthTested":false},{"kind":"sun","color":[1,1,1],"size":24}]"#,
  )
  .unwrap();

  assert_eq!(
    overlays,
    vec![
      RenderOverlay::Lines {
        positions: vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0],
        colors: vec![1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
        is_depth_tested: false,
      },
      RenderOverlay::Sun {
        color: [1.0, 1.0, 1.0],
        size: 24.0,
      },
    ]
  );
}
