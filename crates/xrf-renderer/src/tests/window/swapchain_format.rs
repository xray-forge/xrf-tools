//! Which of a window surface's formats its swapchain takes.

use crate::window::render_window::RenderWindow;

/// The present writes display-encoded eight-bit values, so a surface offering an eight-bit unorm format gets it, the
/// blue-first one where it offers both.
#[test]
fn takes_an_eight_bit_unorm_format_the_surface_offers() {
  let offered = [
    wgpu::TextureFormat::Bgra8UnormSrgb,
    wgpu::TextureFormat::Rgba8Unorm,
    wgpu::TextureFormat::Bgra8Unorm,
  ];

  assert_eq!(
    RenderWindow::choose_format(&offered).unwrap(),
    wgpu::TextureFormat::Bgra8Unorm
  );
  assert_eq!(
    RenderWindow::choose_format(&offered[..2]).unwrap(),
    wgpu::TextureFormat::Rgba8Unorm
  );
}

/// A surface offering only formats the present has no path for (an sRGB one would encode the picture twice, a ten-bit
/// or half-float one would take its dither at the wrong step) is refused, naming them, rather than drawn into wrong.
#[test]
fn refuses_a_surface_offering_no_format_the_present_writes_for() {
  let offered = [wgpu::TextureFormat::Bgra8UnormSrgb, wgpu::TextureFormat::Rgb10a2Unorm];
  let error: String = RenderWindow::choose_format(&offered).unwrap_err().to_string();

  assert!(
    error.contains("Bgra8UnormSrgb") && error.contains("Rgb10a2Unorm"),
    "{error}"
  );
  assert!(RenderWindow::choose_format(&[]).is_err());
}
