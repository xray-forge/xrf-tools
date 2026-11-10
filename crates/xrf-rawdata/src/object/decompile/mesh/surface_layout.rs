use xrf_editable_object::EditableSurface;

use crate::object::decompile::geometry::source::ogf_surface::OgfSurface;
use crate::object::decompile::geometry::welded_face::WeldedFace;

/// The editable surfaces the OGF surfaces' faces land in: one for an OGF surface's single-sided faces and one for its
/// two-sided ones, each only where it has such faces. The export merged surfaces sharing a texture and shader into one
/// child, so a child holding both kinds of face was at least two surfaces.
pub(crate) struct SurfaceLayout {
  pub(crate) surfaces: Vec<EditableSurface>,
  /// The editable surface of each OGF surface's single-sided and two-sided faces, in that order.
  indices: Vec<[Option<usize>; 2]>,
}

impl SurfaceLayout {
  /// What a surface carries the two-sided flag for, when its OGF child also held single-sided faces.
  const TWO_SIDED_SUFFIX: &'static str = "_2sided";

  pub(crate) fn of(surfaces: &[OgfSurface], faces: &[WeldedFace]) -> Self {
    let mut sides: Vec<[bool; 2]> = vec![[false; 2]; surfaces.len()];

    for face in faces {
      sides[face.surface][usize::from(face.is_two_sided)] = true;
    }

    let mut layout: Self = Self {
      surfaces: Vec::new(),
      indices: vec![[None; 2]; surfaces.len()],
    };

    for (index, (surface, present)) in surfaces.iter().zip(sides).enumerate() {
      for is_two_sided in [false, true] {
        if !present[usize::from(is_two_sided)] {
          continue;
        }

        let suffix: &str = if is_two_sided && present[0] {
          Self::TWO_SIDED_SUFFIX
        } else {
          ""
        };
        let name: String = layout.get_unique_name(&format!("{}{suffix}", Self::get_texture_stem(&surface.texture)));

        layout.indices[index][usize::from(is_two_sided)] = Some(layout.surfaces.len());
        layout.surfaces.push(EditableSurface {
          name,
          shader: surface.shader.clone(),
          compiler_shader: surface.dress.compiler_shader.clone(),
          game_material: surface.dress.game_material.clone(),
          texture: surface.texture.clone(),
          vmap: String::from(EditableSurface::UV_MAP_NAME),
          flags: if is_two_sided {
            EditableSurface::TWO_SIDED_FLAG
          } else {
            0
          },
          fvf: EditableSurface::POSITION_NORMAL_UV_FVF,
          texture_coordinate_count: 1,
        });
      }
    }

    layout
  }

  /// The editable surface a face of OGF surface `surface` lands in.
  pub(crate) fn get_surface(&self, surface: usize, is_two_sided: bool) -> usize {
    self.indices[surface][usize::from(is_two_sided)].expect("every face's side was laid out a surface")
  }

  /// The texture's file name without its folders, which names a surface the OGF kept no name for.
  fn get_texture_stem(texture: &str) -> &str {
    texture.rsplit(['\\', '/']).next().unwrap_or(texture)
  }

  /// `name`, or it with the first free `_n` where a surface already has it.
  fn get_unique_name(&self, name: &str) -> String {
    let is_taken = |candidate: &str| self.surfaces.iter().any(|surface| surface.name == candidate);

    if !is_taken(name) {
      return String::from(name);
    }

    (1..)
      .map(|number| format!("{name}_{number}"))
      .find(|candidate| !is_taken(candidate))
      .expect("some numbered name is free")
  }
}
