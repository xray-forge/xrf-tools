use xrf_dds::{DdsEncoding, DdsMipChain, DdsMipmaps, ImageFormat, Quality, Rgba, RgbaImage};
use xrf_editable_object::EditableOccluder;
use xrf_error::XrfResult;
use xrf_material::{XraySurfaceDeclaration, XraySurfaceDescriptor, XraySurfaceDraw};
use xrf_visual::VisualDescription;

/// How a scene draws an occluder object, as the Level Editor does (`CEditableObject::Render`): every face one
/// translucent fill of the occluder's colour in place of its surfaces, which the game never draws.
// todo: Draw an occluder's edges over its fill in the editor's edge colour (`0x40B64646` for sight, `0xFF000000` for
// sound); the renderer has no wire draw for one model alone.
pub(crate) struct SceneOccluderLook;

impl SceneOccluderLook {
  /// The texture a sight occluder's fill is drawn with, a reference no game data spells.
  pub(crate) const HOM_TEXTURE: &'static str = "$editor\\occluder_hom";

  /// The texture a sound occluder's fill is drawn with, a reference no game data spells.
  pub(crate) const SOUND_TEXTURE: &'static str = "$editor\\occluder_sound";

  /// The editor's fill of a sight occluder, `0xA0FFFFFF`, as RGBA.
  pub(crate) const HOM_FILL: [u8; 4] = [0xff, 0xff, 0xff, 0xa0];

  /// The editor's fill of a sound occluder, `0xA00000FF`, as RGBA.
  pub(crate) const SOUND_FILL: [u8; 4] = [0x00, 0x00, 0xff, 0xa0];

  /// Texels along a fill texture's edge: one block, the least a block-compressed texture holds.
  const FILL_SIZE: u32 = DdsEncoding::BLOCK_SIZE;

  /// Dresses every submesh of `description` in the `occluder`'s fill texture and returns each one's surface: blended,
  /// every texel kept.
  pub(crate) fn dress(occluder: EditableOccluder, description: &mut VisualDescription) -> Vec<XraySurfaceDescriptor> {
    let texture: String = String::from(Self::get_texture(occluder));
    let fill: XraySurfaceDescriptor = XraySurfaceDescriptor {
      draw: XraySurfaceDraw::Blended { reference: 0 },
      ..XraySurfaceDescriptor::opaque(None, XraySurfaceDeclaration::Undeclared)
    }
    .resolved_from(&texture, std::slice::from_ref(&texture));

    for submesh in &mut description.submeshes {
      submesh.texture_name = Some(texture.clone());
    }

    vec![fill; description.submeshes.len()]
  }

  /// The DDS bytes of the fill texture `reference`, one flat block of its occluder's colour; none for any other
  /// reference, which the game data answers.
  ///
  /// # Errors
  ///
  /// Returns an error when the fill does not encode.
  pub(crate) fn read_texture(reference: &str) -> XrfResult<Option<Vec<u8>>> {
    let fill: [u8; 4] = match reference {
      Self::HOM_TEXTURE => Self::HOM_FILL,
      Self::SOUND_TEXTURE => Self::SOUND_FILL,
      _ => return Ok(None),
    };
    let base: RgbaImage = RgbaImage::from_pixel(Self::FILL_SIZE, Self::FILL_SIZE, Rgba(fill));

    DdsEncoding::new(ImageFormat::BC3RgbaUnorm, Quality::Fast)
      .encode(&DdsMipChain::build(base, DdsMipmaps::Disabled)?)?
      .write_to_bytes()
      .map(Some)
  }

  /// The texture an `occluder`'s fill is drawn with.
  const fn get_texture(occluder: EditableOccluder) -> &'static str {
    match occluder {
      EditableOccluder::Hom => Self::HOM_TEXTURE,
      EditableOccluder::Sound => Self::SOUND_TEXTURE,
    }
  }
}
