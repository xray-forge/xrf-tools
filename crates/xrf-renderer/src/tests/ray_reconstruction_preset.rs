use xrf_dlss::DlssRayPreset;

use crate::contract::render_ray_reconstruction_preset::RenderRayReconstructionPreset;

#[test]
fn leaves_nvidias_pick_to_nvidia_outside_the_physical_light() {
  assert_eq!(
    RenderRayReconstructionPreset::Default.to_dlss(false),
    DlssRayPreset::Default
  );
}

#[test]
fn picks_d_for_nvidias_pick_under_the_physical_light() {
  assert_eq!(RenderRayReconstructionPreset::Default.to_dlss(true), DlssRayPreset::D);
}

#[test]
fn keeps_a_model_asked_for_under_either_light() {
  for is_light_physical in [false, true] {
    assert_eq!(
      RenderRayReconstructionPreset::D.to_dlss(is_light_physical),
      DlssRayPreset::D
    );
    assert_eq!(
      RenderRayReconstructionPreset::E.to_dlss(is_light_physical),
      DlssRayPreset::E
    );
    assert_eq!(
      RenderRayReconstructionPreset::F.to_dlss(is_light_physical),
      DlssRayPreset::F
    );
  }
}
