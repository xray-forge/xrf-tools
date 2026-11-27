use xrf_renderer::{RenderLayerTextureId, RenderLayerTextures};

use crate::paint::developer_paint::DeveloperPaint;

fn create_delta(size: [usize; 2]) -> Vec<egui::epaint::ImageDelta> {
  vec![egui::epaint::ImageDelta::full(
    egui::ColorImage::filled(size, egui::Color32::WHITE),
    egui::TextureOptions::LINEAR,
  )]
}

#[test]
fn textures_come_in_id_order_with_managed_and_user_apart() {
  let mut delta: egui::TexturesDelta = egui::TexturesDelta::default();

  delta.set.insert(egui::TextureId::User(0), create_delta([1, 1]).into());
  delta
    .set
    .insert(egui::TextureId::Managed(1), create_delta([2, 2]).into());
  delta
    .set
    .insert(egui::TextureId::Managed(0), create_delta([4, 4]).into());
  delta.free.insert(egui::TextureId::Managed(3));

  let textures: RenderLayerTextures = DeveloperPaint::create_textures(delta);
  let ids: Vec<RenderLayerTextureId> = textures.set.iter().map(|texture| texture.id).collect();

  assert_eq!(
    ids,
    [
      RenderLayerTextureId(0),
      RenderLayerTextureId(1),
      RenderLayerTextureId(2)
    ]
  );
  assert_eq!(textures.set[0].width, 4);
  assert_eq!(textures.set[0].pixels.len(), 4 * 4 * 4);
  assert_eq!(textures.free, [RenderLayerTextureId(6)]);
}
