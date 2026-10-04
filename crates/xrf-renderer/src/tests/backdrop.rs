use crate::contract::render_color::RenderColor;
use crate::contract::render_page_backdrop::RenderPageBackdrop;
use crate::contract::render_page_wash::RenderPageWash;
use crate::contract::render_rect::RenderRect;
use crate::pass::backdrop_uniform::BackdropUniform;

fn wash(width: u32, height: u32, angle: f32) -> RenderPageBackdrop {
  RenderPageBackdrop {
    color: RenderColor { r: 255, g: 0, b: 0 },
    wash: Some(RenderPageWash {
      rect: RenderRect {
        x: 10,
        y: 20,
        width,
        height,
      },
      angle,
      from: [0.0, 0.0, 1.0, 0.5],
      to: [0.0, 1.0, 0.0, 0.25],
    }),
  }
}

#[test]
fn paints_a_plain_colour_without_a_wash() {
  let uniform: BackdropUniform = BackdropUniform::new(&RenderPageBackdrop {
    color: RenderColor { r: 255, g: 0, b: 51 },
    wash: None,
  });

  assert_eq!(uniform.color, [1.0, 0.0, 0.2, 1.0]);
  assert!(!uniform.is_washed());
}

// CSS lays a 135° gradient from the box's top left corner to its bottom right, as long as the box projects onto it.
#[test]
fn lays_the_gradient_line_as_css_does() {
  let uniform: BackdropUniform = BackdropUniform::new(&wash(200, 100, 135.0));
  let expected: f32 = 300.0 * std::f32::consts::FRAC_1_SQRT_2;

  assert!(uniform.is_washed());
  assert_eq!(uniform.rect, [10.0, 20.0, 200.0, 100.0]);
  assert!((uniform.line[0] - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-5);
  assert!((uniform.line[1] - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-5);
  assert!((uniform.line[2] - expected).abs() < 1e-3);
  assert_eq!(uniform.first, [0.0, 0.0, 0.5, 0.5]);
  assert_eq!(uniform.last, [0.0, 0.25, 0.0, 0.25]);
}

#[test]
fn paints_no_wash_over_an_empty_box() {
  assert!(!BackdropUniform::new(&wash(0, 100, 135.0)).is_washed());
}
