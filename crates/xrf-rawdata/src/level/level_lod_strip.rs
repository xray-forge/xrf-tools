use xrf_dds::{DdsEncoding, DdsMipChain, DdsMipFilter, DdsMipmaps, ImageFormat, Quality, RgbaImage};
use xrf_error::XrfResult;
use xrf_math::{Aabb, Vector2d, Vector3d};

/// An object's impostor as the editor lays it out (`CEditableObject::GetLODFrame`, `ECore/Editor/EditObjectEditor.cpp`):
/// eight views of it, each a quad standing through the middle of its box turned an eighth of a circle further, drawn
/// from a strip of eight 64 by 64 frames side by side.
pub(crate) struct LevelLodStrip;

impl LevelLodStrip {
  /// `LOD_SAMPLE_COUNT` and `LOD_IMAGE_SIZE`: frames, and the side of one.
  pub(crate) const FRAMES: usize = 8;
  pub(crate) const FRAME_SIZE: u32 = 64;

  /// The strip's size, frames side by side.
  pub(crate) const WIDTH: u32 = Self::FRAME_SIZE * Self::FRAMES as u32;
  pub(crate) const HEIGHT: u32 = Self::FRAME_SIZE;

  /// What the normals' strip's name adds to the colours'.
  pub(crate) const NORMAL_SUFFIX: &'static str = "_nm";

  /// `image`, a strip or an atlas of them, as the Build saves its impostor textures (`BuilderRemote.cpp:1544`): DXT5
  /// with mips, advanced filtering.
  ///
  /// # Errors
  ///
  /// Returns an error when the image does not encode.
  pub(crate) fn encode(image: &RgbaImage) -> XrfResult<Vec<u8>> {
    DdsEncoding::new(ImageFormat::BC3RgbaUnorm, Quality::Normal)
      .encode(&DdsMipChain::build(image, DdsMipmaps::Filtered(DdsMipFilter::Kaiser))?)?
      .write_to_bytes()
  }

  /// The texture the object of `reference` takes its strip from (`CEditableObject::GetLODTextureName`): `lod_` and its
  /// reference, folders joined by `_`, in the `lod` folder `ImageLib.UpdateFileName` takes from its first word.
  pub(crate) fn get_texture_name(reference: &str) -> String {
    format!("lod\\lod_{}", reference.replace('\\', "_"))
  }

  /// The texture coordinates of `frame`'s corners in the strip (`CalculateLODTC`): its cell, half a pixel in from each
  /// edge, corners clockwise from the top left as the quad's are.
  pub(crate) fn get_frame_coordinates(frame: usize) -> [Vector2d; 4] {
    let (half_x, half_y) = (0.5 / Self::WIDTH as f32, 0.5 / Self::HEIGHT as f32);
    let step: f32 = 1.0 / Self::FRAMES as f32;
    let (left, right) = (frame as f32 * step + half_x, (frame + 1) as f32 * step - half_x);
    let (top, bottom) = (half_y, 1.0 - half_y);

    [
      Vector2d::new(left, top),
      Vector2d::new(right, top),
      Vector2d::new(right, bottom),
      Vector2d::new(left, bottom),
    ]
  }

  /// What the first frame's quad `corners`, in the object's own space, was stood from: the box's middle, its widest
  /// half side across and its half height, read back from the quad's top left and bottom right corners.
  pub(crate) fn get_first_frame_extent(corners: &[Vector3d; 4]) -> (Vector3d, f32, f32) {
    let (offset_x, offset_y) = Self::get_corner_offsets();
    let (top_left, bottom_right) = (corners[0], corners[2]);

    (
      Vector3d::new(
        f32::midpoint(top_left.x, bottom_right.x),
        f32::midpoint(top_left.y, bottom_right.y),
        f32::midpoint(top_left.z, bottom_right.z),
      ),
      (bottom_right.x - top_left.x) / (2.0 * (1.0 - offset_x)),
      (top_left.y - bottom_right.y) / (2.0 * (1.0 - offset_y)),
    )
  }

  /// How far `LOD_pos` stands its corners in from the unit square's, a pixel of the strip across and up.
  fn get_corner_offsets() -> (f32, f32) {
    (1.0 / Self::WIDTH as f32, 1.0 / Self::HEIGHT as f32)
  }

  /// `frame`'s quad around an object of `bounds`, in the object's own space: the unit square `LOD_pos`, a pixel in from
  /// each edge, turned about y by the frame's angle, stretched over the box's widest half side across and its half
  /// height up, and moved to its middle.
  pub(crate) fn get_frame_corners(frame: usize, bounds: &Aabb) -> [Vector3d; 4] {
    let (offset_x, offset_y) = Self::get_corner_offsets();
    let center: Vector3d = Vector3d::new(
      f32::midpoint(bounds.min.x, bounds.max.x),
      f32::midpoint(bounds.min.y, bounds.max.y),
      f32::midpoint(bounds.min.z, bounds.max.z),
    );
    let half: Vector3d = Vector3d::new(
      (bounds.max.x - bounds.min.x) / 2.0,
      (bounds.max.y - bounds.min.y) / 2.0,
      (bounds.max.z - bounds.min.z) / 2.0,
    );
    let across: f32 = half.x.max(half.z);
    let angle: f32 = frame as f32 * std::f32::consts::TAU / Self::FRAMES as f32;
    let (sin, cos) = angle.sin_cos();

    [
      (-1.0 + offset_x, 1.0 - offset_y),
      (1.0 - offset_x, 1.0 - offset_y),
      (1.0 - offset_x, -1.0 + offset_y),
      (-1.0 + offset_x, -1.0 + offset_y),
    ]
    .map(|(x, y)| {
      // `rotateY(-angle)`, then the box's scale and middle; the square stands in z = 0.
      Vector3d::new(
        x * cos * across + center.x,
        y * half.y + center.y,
        x * sin * across + center.z,
      )
    })
  }
}
