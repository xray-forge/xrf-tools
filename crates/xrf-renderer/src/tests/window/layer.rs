use crate::contract::layer::render_layer::RenderLayer;
use crate::contract::layer::render_layer_mesh::RenderLayerMesh;
use crate::contract::layer::render_layer_texture::RenderLayerTexture;
use crate::contract::layer::render_layer_texture_id::RenderLayerTextureId;
use crate::contract::layer::render_layer_textures::RenderLayerTextures;
use crate::viewport::layer::layer_region::LayerRegion;
use crate::viewport::layer::viewport_layer::ViewportLayer;

const ATLAS: RenderLayerTextureId = RenderLayerTextureId(0);

fn create_set(origin: Option<[u32; 2]>, width: u32, height: u32, value: u8) -> RenderLayerTexture {
  RenderLayerTexture {
    id: ATLAS,
    origin,
    width,
    height,
    pixels: vec![value; width as usize * height as usize * 4],
    is_nearest: false,
  }
}

fn change(layer: &mut ViewportLayer, set: Vec<RenderLayerTexture>, free: Vec<RenderLayerTextureId>) {
  layer.change_textures(RenderLayerTextures { set, free });
}

#[test]
fn region_patches_the_image_and_widens_what_changed() {
  let mut layer: ViewportLayer = ViewportLayer::default();

  change(&mut layer, vec![create_set(None, 4, 4, 0)], vec![]);
  layer.changed.clear();
  change(
    &mut layer,
    vec![create_set(Some([1, 2]), 2, 1, 9), create_set(Some([3, 0]), 1, 1, 7)],
    vec![],
  );

  let pixels: &[u8] = &layer.images[&ATLAS].pixels;

  // Row 2, columns 1 and 2 hold the first region; row 0, column 3 the second.
  assert_eq!(&pixels[(2 * 4 + 1) * 4..(2 * 4 + 3) * 4], &[9; 8]);
  assert_eq!(&pixels[3 * 4..4 * 4], &[7; 4]);
  assert_eq!(pixels.iter().filter(|value| **value != 0).count(), 12);
  assert_eq!(layer.changed[&ATLAS], LayerRegion::new(1, 0, 3, 3));
}

#[test]
fn changes_that_do_not_fit_are_left_out() {
  let mut layer: ViewportLayer = ViewportLayer::default();

  change(&mut layer, vec![create_set(Some([0, 0]), 1, 1, 1)], vec![]);
  assert!(layer.images.is_empty());

  change(&mut layer, vec![create_set(None, 2, 2, 0)], vec![]);
  layer.changed.clear();
  change(
    &mut layer,
    vec![
      create_set(Some([1, 1]), 2, 1, 5),
      RenderLayerTexture {
        pixels: vec![1; 3],
        ..create_set(Some([0, 0]), 1, 1, 0)
      },
    ],
    vec![],
  );

  assert!(layer.images[&ATLAS].pixels.iter().all(|value| *value == 0));
  assert!(layer.changed.is_empty());
}

#[test]
fn released_gpu_takes_every_image_and_the_layer_again() {
  let mut layer: ViewportLayer = ViewportLayer::default();

  change(&mut layer, vec![create_set(None, 8, 2, 0)], vec![]);
  layer.set(RenderLayer::default());
  layer.changed.clear();
  layer.is_layer_changed = false;
  layer.release_gpu();

  assert!(layer.is_layer_changed);
  assert_eq!(layer.changed[&ATLAS], LayerRegion::new(0, 0, 8, 2));
}

#[test]
fn layer_is_sent_once_it_holds_a_mesh() {
  let mut layer: ViewportLayer = ViewportLayer::default();

  change(&mut layer, vec![create_set(None, 1, 1, 0)], vec![]);
  assert!(!layer.is_sent(), "a texture alone draws nothing");

  layer.set(RenderLayer {
    meshes: vec![RenderLayerMesh::default()],
  });
  assert!(layer.is_sent());
  assert!(!layer.is_drawn(), "sent, it is drawn once uploaded");
}

#[test]
fn freed_texture_goes_with_its_image() {
  let mut layer: ViewportLayer = ViewportLayer::default();

  change(&mut layer, vec![create_set(None, 1, 1, 0)], vec![ATLAS]);

  assert!(layer.images.is_empty());
  assert!(layer.changed.contains_key(&ATLAS));
}
