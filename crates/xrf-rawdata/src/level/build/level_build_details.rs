use xrf_dds::{DdsEncoding, DdsMipChain, DdsMipFilter, DdsMipmaps, ImageFormat, Quality, RgbaImage};
use xrf_editable_level::{EditableDetailObject, EditableDetailObjectTool};
use xrf_editable_object::EditableObjectFile;
use xrf_error::{XrfError, XrfResult};
use xrf_level::{DetailModel, DetailVertex, LevelDetailsFile, LevelDetailsHeader, LevelDetailsLight, LevelDetailsSlot};
use xrf_utils::narrow_format_size;

use crate::level::build::level_build_atlas::LevelBuildAtlas;
use crate::level::build::level_build_atlas_place::LevelBuildAtlasPlace;
use crate::level::build::level_build_library::LevelBuildLibrary;
use crate::level::build::level_build_textures::LevelBuildTextures;
use crate::object::compile::detail::detail_model_builder::DetailModelBuilder;
use crate::object::detail_parameters::DetailParameters;

/// What the Build makes of the detail object tool, `EDetailManager::Export`: `build.details`, which the detail lighter
/// turns into `level.details`, and the atlas of the planted objects' textures, `build_details.dds`.
#[derive(Clone, Debug, PartialEq)]
pub struct LevelBuildDetails {
  /// The planted objects, their coordinates moved into the atlas, and the slots renumbered with their light zeroed.
  pub file: LevelDetailsFile,
  /// `build_details.dds`, DXT1 with one bit of alpha and mipmaps.
  pub atlas: Vec<u8>,
  /// The atlas's size, and how many textures it merges.
  pub atlas_size: (u32, u32),
  pub textures: usize,
}

impl LevelBuildDetails {
  pub const FILE_NAME: &'static str = "build.details";

  /// The atlas's file, and the texture name each object draws with.
  pub const ATLAS_FILE_NAME: &'static str = "build_details.dds";
  pub const ATLAS_TEXTURE: &'static str = "build_details";

  /// The details of `tool`, its objects read from `library` and their textures from `textures` (`$game_textures$`).
  ///
  /// # Errors
  ///
  /// Returns an error, as the Build stops, when no slot plants anything, a slot names an object the tool lacks, an
  /// object or texture does not read, or the textures do not fit the largest atlas.
  pub(crate) fn export(
    tool: &EditableDetailObjectTool,
    library: &mut LevelBuildLibrary<'_>,
    textures: &LevelBuildTextures<'_>,
  ) -> XrfResult<Self> {
    let renumbered: Vec<Option<u8>> = Self::renumber(tool)?;
    let planted: Vec<&EditableDetailObject> = tool
      .objects
      .iter()
      .zip(&renumbered)
      .filter_map(|(object, number)| number.map(|_| object))
      .collect();

    if planted.is_empty() {
      return Err(XrfError::new_invalid_error(
        "The detail tool plants nothing, so the Build has no texture to merge into its atlas",
      ));
    }

    library.load(planted.iter().map(|object| object.reference.as_str()))?;

    let models: Vec<DetailModel> = planted
      .iter()
      .map(|object| Self::compile(object, library))
      .collect::<XrfResult<_>>()?;
    let mut names: Vec<String> = models.iter().map(|model| model.texture.clone()).collect();

    names.sort_unstable();
    names.dedup();

    let images: Vec<RgbaImage> = names
      .iter()
      .map(|name| textures.read("Detail texture", name))
      .collect::<XrfResult<_>>()?;
    let atlas: LevelBuildAtlas = LevelBuildAtlas::merge(
      &images.iter().map(std::slice::from_ref).collect::<Vec<_>>(),
      &LevelBuildAtlas::DETAIL_SIZES,
    )
    .ok_or_else(|| {
      let (width, height) = LevelBuildAtlas::DETAIL_SIZES.largest;

      XrfError::new_invalid_error(format!(
        "The detail objects' {} textures do not fit an atlas {width} by {height}",
        names.len()
      ))
    })?;
    let image: &RgbaImage = &atlas.layers[0];
    // Each model's texture is among the sorted names, which the atlas places in their order.
    let objects: Vec<DetailModel> = models
      .into_iter()
      .map(|model| {
        let index: usize = names
          .binary_search(&model.texture)
          .map_err(|_| XrfError::new_invalid_error(format!("Detail texture '{}' was not merged", model.texture)))?;

        Ok(Self::remap(model, &atlas.places[index]))
      })
      .collect::<XrfResult<_>>()?;
    let slots: Vec<[u32; EditableDetailObjectTool::SLOT_WORDS]> = tool
      .slots
      .iter()
      .map(|slot| {
        (0..LevelDetailsSlot::CORNERS).fold(
          LevelDetailsSlot::with_light(*slot, LevelDetailsLight::UNLIT),
          |slot, corner| match LevelDetailsSlot::get_object(&slot, corner) {
            Some(object) => LevelDetailsSlot::with_object(
              slot,
              corner,
              renumbered[usize::from(object)].unwrap_or(LevelDetailsSlot::EMPTY_ID),
            ),
            None => slot,
          },
        )
      })
      .collect();

    Ok(Self {
      file: LevelDetailsFile {
        header: LevelDetailsHeader {
          version: LevelDetailsFile::CURRENT_VERSION,
          object_count: narrow_format_size(objects.len(), "A details file's object count")?,
          offset_x: tool.header.offset_x,
          offset_z: tool.header.offset_z,
          size_x: tool.header.size_x,
          size_z: tool.header.size_z,
        },
        objects,
        slots,
      },
      atlas: Self::encode(image)?,
      atlas_size: image.dimensions(),
      textures: names.len(),
    })
  }

  /// `image` encoded as the editor saves the atlas, `tfADXT1`: DXT1 with mips whose cut-out pixels stay transparent,
  /// or each card would draw black around its leaves.
  ///
  /// # Errors
  ///
  /// Returns an error when the image does not encode.
  pub(crate) fn encode(image: &RgbaImage) -> XrfResult<Vec<u8>> {
    DdsEncoding::new(ImageFormat::BC1RgbaUnorm, Quality::Normal)
      .with_cut_out()
      .encode(&DdsMipChain::build(image, DdsMipmaps::Filtered(DdsMipFilter::Kaiser))?)?
      .write_to_bytes()
  }

  /// Each tool object's number in the export: the planted ones counted in the tool's order, the rest none.
  fn renumber(tool: &EditableDetailObjectTool) -> XrfResult<Vec<Option<u8>>> {
    let mut is_planted: Vec<bool> = vec![false; tool.objects.len()];

    for slot in &tool.slots {
      for corner in 0..LevelDetailsSlot::CORNERS {
        if let Some(object) = LevelDetailsSlot::get_object(slot, corner) {
          *is_planted.get_mut(usize::from(object)).ok_or_else(|| {
            XrfError::new_invalid_error(format!(
              "A detail slot plants object {object}, past the tool's {}",
              tool.objects.len()
            ))
          })? = true;
        }
      }
    }

    let mut next: u8 = 0;

    Ok(
      is_planted
        .into_iter()
        .map(|is_planted| {
          is_planted.then(|| {
            next += 1;
            next - 1
          })
        })
        .collect(),
    )
  }

  /// The model `EDetail` exports of `object`'s library object, at the object's scale range and flags.
  fn compile(object: &EditableDetailObject, library: &LevelBuildLibrary<'_>) -> XrfResult<DetailModel> {
    let file: &EditableObjectFile = library
      .get(&object.reference)
      .ok_or_else(|| XrfError::new_invalid_error(format!("Detail object '{}' was not read", object.reference)))?;
    let parameters: DetailParameters = DetailParameters {
      flags: object.flags,
      min_scale: object.min_scale,
      max_scale: object.max_scale,
    };

    DetailModelBuilder::build_model(file, &parameters)
      .map(|(model, _)| model)
      .map_err(|error| XrfError::new_invalid_error(format!("Detail object '{}': {error}", object.reference)))
  }

  /// `model` drawing from its texture's `place` in the atlas, named [`Self::ATLAS_TEXTURE`].
  fn remap(model: DetailModel, place: &LevelBuildAtlasPlace) -> DetailModel {
    DetailModel {
      texture: String::from(Self::ATLAS_TEXTURE),
      vertices: model
        .vertices
        .iter()
        .map(|vertex| {
          let (u, v) = place.remap(vertex.u, vertex.v);

          DetailVertex {
            position: vertex.position,
            u,
            v,
          }
        })
        .collect(),
      ..model
    }
  }
}
