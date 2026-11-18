use crate::window::render_window::RenderWindow;

// The present writes display-encoded eight-bit values.
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

// sRGB would encode twice; ten-bit or half-float would dither at the wrong step.
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
