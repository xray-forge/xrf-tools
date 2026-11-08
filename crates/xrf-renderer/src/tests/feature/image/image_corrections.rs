use glam::Vec4;

use crate::contract::feature::image::display::render_image_corrections::RenderImageCorrections;
use crate::feature::image::present::present_frame::PresentFrame;
use crate::feature::image::present::present_uniform::PresentUniform;

// The present divides by the gamma and multiplies by the rest: each reaches it within the engine's console ranges, and
// one that is no finite number corrects nothing.
#[test]
fn holds_the_corrections_within_the_engines_ranges() {
  let present = |corrections: RenderImageCorrections| PresentUniform::new(PresentFrame::default(), &corrections);

  let held: PresentUniform = present(RenderImageCorrections {
    exposure: 9.0,
    gamma: 0.0,
    saturation: -1.0,
    grading: [2.0, -1.0, 0.5],
  });

  assert_eq!(held.corrections, Vec4::new(4.0, 0.5, 0.0, 0.0));
  assert_eq!(held.grading, Vec4::new(1.0, 0.0, 0.5, 0.0));

  let neutral: PresentUniform = present(RenderImageCorrections {
    exposure: f32::NAN,
    gamma: f32::INFINITY,
    saturation: f32::NEG_INFINITY,
    grading: [f32::NAN; 3],
  });

  assert_eq!(neutral.corrections, Vec4::new(1.0, 1.0, 1.0, 0.0));
  assert_eq!(neutral.grading, Vec4::ZERO);

  let kept: RenderImageCorrections = RenderImageCorrections {
    exposure: 1.2,
    gamma: 1.4,
    saturation: 1.05,
    grading: [0.5, 0.25, 0.0],
  };

  assert_eq!(kept.clamped(), kept);
}
