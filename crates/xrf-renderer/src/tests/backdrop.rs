use glam::Vec4;

use crate::contract::render_color::RenderColor;
use crate::contract::render_page_backdrop::RenderPageBackdrop;
use crate::contract::render_page_plane::RenderPagePlane;
use crate::contract::render_page_wash::RenderPageWash;
use crate::contract::render_rect::RenderRect;
use crate::pass::backdrop_uniform::BackdropUniform;

fn plane(x: i32, width: u32, is_washed: bool) -> RenderPagePlane {
  RenderPagePlane {
    rect: RenderRect {
      x,
      y: 0,
      width,
      height: 100,
    },
    radii: [8.0, 8.0, 0.0, 0.0],
    color: RenderColor { r: 255, g: 0, b: 51 },
    is_washed,
  }
}

fn wash(width: u32, height: u32, angle: f32) -> RenderPageBackdrop {
  RenderPageBackdrop {
    planes: vec![plane(0, 400, true)],
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
fn clears_to_one_plane_without_a_wash() {
  let uniform: BackdropUniform = BackdropUniform::new(&RenderPageBackdrop {
    planes: vec![plane(0, 400, false)],
    wash: None,
  });

  assert_eq!(uniform.colors[0], Vec4::new(1.0, 0.0, 0.2, 0.0));
  assert_eq!(uniform.get_clear().b, f64::from(0.2_f32));
  assert!(uniform.is_flat());
}

#[test]
fn clears_to_black_where_the_page_paints_nothing() {
  let uniform: BackdropUniform = BackdropUniform::new(&RenderPageBackdrop::default());

  assert_eq!(uniform.get_clear(), wgpu::Color::BLACK);
  assert!(uniform.is_flat());
}

// The frame around a viewport and the plane it sits on differ, so a clear to either one paints the other wrong.
#[test]
fn paints_each_inner_plane_in_its_own_box() {
  let uniform: BackdropUniform = BackdropUniform::new(&RenderPageBackdrop {
    planes: vec![plane(0, 400, false), plane(40, 300, false)],
    wash: None,
  });

  assert!(!uniform.is_flat());
  assert_eq!(uniform.boxes[1], Vec4::new(40.0, 0.0, 300.0, 100.0));
  assert_eq!(uniform.radii[1], Vec4::new(8.0, 8.0, 0.0, 0.0));
  assert_eq!(uniform.boxes[2], Vec4::ZERO);
}

#[test]
fn keeps_the_outermost_and_the_innermost_planes() {
  let planes: Vec<RenderPagePlane> = (0..6).map(|index| plane(index, 100, false)).collect();
  let uniform: BackdropUniform = BackdropUniform::new(&RenderPageBackdrop { planes, wash: None });
  let kept: Vec<f32> = uniform.boxes.iter().map(|it| it.x).collect();

  assert_eq!(kept, vec![0.0, 3.0, 4.0, 5.0]);
}

// CSS lays a 135° gradient from the box's top left corner to its bottom right, as long as the box projects onto it.
#[test]
fn lays_the_gradient_line_as_css_does() {
  let uniform: BackdropUniform = BackdropUniform::new(&wash(200, 100, 135.0));
  let expected: f32 = 300.0 * std::f32::consts::FRAC_1_SQRT_2;

  assert!(!uniform.is_flat());
  assert_eq!(uniform.colors[0].w, 1.0);
  assert_eq!(uniform.rect, Vec4::new(10.0, 20.0, 200.0, 100.0));
  assert!((uniform.line.x - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-5);
  assert!((uniform.line.y - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-5);
  assert!((uniform.line.z - expected).abs() < 1e-3);
  assert_eq!(uniform.first, Vec4::new(0.0, 0.0, 0.5, 0.5));
  assert_eq!(uniform.last, Vec4::new(0.0, 0.25, 0.0, 0.25));
}

#[test]
fn paints_no_wash_over_an_empty_box() {
  let uniform: BackdropUniform = BackdropUniform::new(&wash(0, 100, 135.0));

  assert_eq!(uniform.colors[0].w, 0.0);
  assert!(uniform.is_flat());
}
