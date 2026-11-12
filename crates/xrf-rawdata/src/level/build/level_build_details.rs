use byteorder::ByteOrder;
use xrf_dds::{DdsEncoding, DdsMipChain, DdsMipFilter, DdsMipmaps, ImageFormat, Quality, RgbaImage};
use xrf_editable_level::{EditableDetailObject, EditableDetailObjectTool};
use xrf_editable_object::EditableObjectFile;
use xrf_error::{XrfError, XrfResult};
use xrf_level::{DetailModel, DetailVertex, LevelDetailsFile, LevelDetailsHeader, LevelDetailsLight, LevelDetailsSlot};

use crate::level::build::level_build_atlas::{LevelBuildAtlas, LevelBuildAtlasPlace};
use crate::level::build::level_build_library::LevelBuildLibrary;
use crate::level::build::level_build_textures::LevelBuildTextures;
use crate::object::compile::detail::detail_model_builder::DetailModelBuilder;
use crate::object::detail_parameters::DetailParameters;

/// What the Build makes of the detail object tool (`EDetailManager::Export`, `LevelEditor/Edit/ESceneDOTools.cpp:500`):
/// `build.details`, which the detail lighter turns into `level.details`, and the atlas of the planted objects' textures
/// beside it, `build_details.dds`.
#[derive(Clone, Debug, PartialEq)]
pub struct LevelBuildDetails {
  /// `build.details`: the planted objects, each compiled from its library object with its coordinates moved into the
  /// atlas, and the slots renumbered with their light zeroed.
  pub file: LevelDetailsFile,
  /// `build_details.dds`, DXT1 with its one bit of alpha and mipmaps.
  pub atlas: Vec<u8>,
  /// The atlas's size, and how many textures it merges.
  pub atlas_size: (u32, u32),
  pub textures: usize,
}

impl LevelBuildDetails {
  /// The file the Build writes the details into.
  pub const FILE_NAME: &'static str = "build.details";

  /// The atlas's file beside it, and the texture name each object draws with.
  pub const ATLAS_FILE_NAME: &'static str = "build_details.dds";
  pub const ATLAS_TEXTURE: &'static str = "build_details";

  /// The details of `tool`, its objects read from `library` and their textures from `textures` (`$game_textures$`).
  ///
  /// # Errors
  ///
  /// Returns an error, as the Build stops, where no slot plants anything, so there is no texture to merge; a slot
  /// names an object the tool does not hold; an object does not read or is no detail object; a texture does not read;
  /// or the textures do not fit the largest atlas.
  pub(crate) fn export<T: ByteOrder>(
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

    library.load::<T>(planted.iter().map(|object| object.reference.as_str()))?;

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
    let chain: DdsMipChain = DdsMipChain::build(image, DdsMipmaps::Filtered(DdsMipFilter::Kaiser))?;

    Ok(Self {
      file: LevelDetailsFile {
        header: LevelDetailsHeader {
          version: LevelDetailsFile::CURRENT_VERSION,
          object_count: objects.len() as u32,
          offset_x: tool.header.offset_x,
          offset_z: tool.header.offset_z,
          size_x: tool.header.size_x,
          size_z: tool.header.size_z,
        },
        objects,
        slots,
      },
      // `tfADXT1` (`ESceneDOTools.cpp:542`): DXT1 whose cut-out pixels stay transparent, or each card draws its clear
      // around the leaves black.
      atlas: DdsEncoding::new(ImageFormat::BC1RgbaUnorm, Quality::Normal)
        .with_cut_out()
        .encode(&chain)?
        .write_to_bytes()?,
      atlas_size: image.dimensions(),
      textures: names.len(),
    })
  }

  /// Each of the tool's objects' number in the export: the planted ones counted in the tool's order, the rest none.
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
