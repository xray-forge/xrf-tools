use std::path::PathBuf;

use byteorder::ByteOrder;
use xrf_editable_object::EditableObjectFile;
use xrf_error::XrfResult;
use xrf_job::JobHandle;
use xrf_level::DetailModel;
use xrf_math::Vector2d;

use crate::level::build::level_build_atlas_place::LevelBuildAtlasPlace;
use crate::level::details::level_detail_match::LevelDetailMatch;
use crate::level::level_object_library::LevelObjectLibrary;
use crate::level::level_object_library_entry::LevelObjectLibraryEntry;
use crate::object::compile::detail::detail_model_builder::DetailModelBuilder;
use crate::object::detail_parameters::DetailParameters;

/// The library detail objects, each compiled as `EDetail` exports it, that a level's detail models are matched
/// against. Only each root's `detail\` folder is read, where the Level Editor keeps them.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct LevelDetailLibrary {
  /// Each object by its reference, in the order of their paths.
  sources: Vec<(String, DetailModel)>,
}

impl LevelDetailLibrary {
  /// The folder of an object library root the Level Editor's detail objects stand in.
  pub(crate) const DIRECTORY: &'static str = "detail";

  /// How far a remapped texture coordinate may stand from the stored one, past single-precision rounding.
  const REMAP_TOLERANCE: f32 = 1e-5;

  /// How little a source's coordinates may span on an axis and still give that axis a scale.
  const SPAN_EPSILON: f32 = 1e-6;

  /// A library of `sources`, each a detail object's reference and its model as `EDetail` exports it, in the order
  /// [`Self::find`] tries them.
  #[cfg(test)]
  pub(crate) fn new(sources: Vec<(String, DetailModel)>) -> Self {
    Self { sources }
  }

  /// The detail objects of `library`, those under each root's [`Self::DIRECTORY`], each compiled as `EDetail` exports
  /// it; one that does not read or compile is recorded in `library` and matches nothing. The walk stops once `job` is
  /// cancelled.
  pub(crate) fn read_from_library<T: ByteOrder>(library: &mut LevelObjectLibrary, job: &JobHandle) -> XrfResult<Self> {
    let mut detail: Self = Self::default();
    let mut uncompiled: Vec<(PathBuf, String)> = Vec::new();

    library.read_each::<T>(job, Self::is_detail, |entry, object| {
      detail.add(entry, &object, &mut uncompiled);

      Ok(())
    })?;

    Self::record_uncompiled(library, uncompiled);

    Ok(detail)
  }

  /// Whether `entry` stands among the detail objects, under its root's [`Self::DIRECTORY`].
  pub(crate) fn is_detail(entry: &LevelObjectLibraryEntry) -> bool {
    entry.is_under(Self::DIRECTORY)
  }

  /// Adds `object`, read from `entry`, compiled as `EDetail` exports it; one that does not compile goes to
  /// `uncompiled` with why, for [`Self::record_uncompiled`] once the walk is done.
  pub(crate) fn add(
    &mut self,
    entry: &LevelObjectLibraryEntry,
    object: &EditableObjectFile,
    uncompiled: &mut Vec<(PathBuf, String)>,
  ) {
    match DetailModelBuilder::build_model(object, &DetailParameters::default()) {
      Ok((model, _)) => self.sources.push((entry.reference.clone(), model)),
      Err(error) => uncompiled.push((
        entry.path.clone(),
        format!("it did not compile as a detail object: {error}"),
      )),
    }
  }

  /// Records in `library` each detail object [`Self::add`] could not compile, which then matches nothing.
  pub(crate) fn record_uncompiled(library: &mut LevelObjectLibrary, uncompiled: Vec<(PathBuf, String)>) {
    for (path, cause) in uncompiled {
      library.record_unread(&path, cause);
    }
  }

  /// The first object drawing `stored`: the same positions and indices, its texture coordinates `stored`'s before
  /// `MergedTextureRemapUV` moved them into the level's atlas.
  pub(crate) fn find(&self, stored: &DetailModel) -> Option<LevelDetailMatch<'_>> {
    self.sources.iter().find_map(|(reference, source)| {
      let is_same_shape: bool = source.indices == stored.indices
        && source.vertices.len() == stored.vertices.len()
        && source
          .vertices
          .iter()
          .zip(&stored.vertices)
          .all(|(source, stored)| source.position == stored.position);

      if is_same_shape {
        Self::fit(reference, source, stored)
      } else {
        None
      }
    })
  }

  /// The match of `source` where `stored`'s coordinates are its own remapped into an atlas, `scale * source + offset`
  /// on each axis, `u` and `v` swapped where the texture was turned to fit, as [`LevelBuildAtlasPlace::remap`] moves
  /// them: the remap fitted from the extreme coordinates and checked on every vertex, unturned first.
  fn fit<'a>(reference: &'a str, source: &'a DetailModel, stored: &DetailModel) -> Option<LevelDetailMatch<'a>> {
    [false, true].into_iter().find_map(|is_rotated| {
      let pairs = |axis: usize| -> Vec<LevelDetailRemapPair> {
        source
          .vertices
          .iter()
          .zip(&stored.vertices)
          .map(|(source, stored)| {
            let from: [f32; 2] = if is_rotated {
              [source.v, source.u]
            } else {
              [source.u, source.v]
            };

            LevelDetailRemapPair {
              from: from[axis],
              to: [stored.u, stored.v][axis],
            }
          })
          .collect()
      };
      let u: LevelDetailAxisRemap = Self::fit_axis(&pairs(0))?;
      let v: LevelDetailAxisRemap = Self::fit_axis(&pairs(1))?;
      let place: Option<LevelBuildAtlasPlace> = match (u, v) {
        (
          LevelDetailAxisRemap::Line {
            scale: u_scale,
            offset: u_offset,
          },
          LevelDetailAxisRemap::Line {
            scale: v_scale,
            offset: v_offset,
          },
        ) => Some(LevelBuildAtlasPlace {
          offset: Vector2d::new(u_offset, v_offset),
          scale: Vector2d::new(u_scale, v_scale),
          is_rotated,
        }),
        _ => None,
      };

      Some(LevelDetailMatch {
        reference,
        source,
        place,
      })
    })
  }

  /// The one remap carrying every pair, fitted from the pairs of the lowest and highest source coordinate; a source
  /// that spans nothing is carried by a shift alone.
  fn fit_axis(pairs: &[LevelDetailRemapPair]) -> Option<LevelDetailAxisRemap> {
    let Some(first) = pairs.first() else {
      return Some(LevelDetailAxisRemap::Shift { offset: 0.0 });
    };
    let (low, high) = pairs.iter().fold((first, first), |(low, high), pair| {
      (
        if pair.from < low.from { pair } else { low },
        if pair.from > high.from { pair } else { high },
      )
    });
    let span: f32 = high.from - low.from;
    let remap: LevelDetailAxisRemap = if span.abs() < Self::SPAN_EPSILON {
      LevelDetailAxisRemap::Shift {
        offset: low.to - low.from,
      }
    } else {
      let scale: f32 = (high.to - low.to) / span;

      LevelDetailAxisRemap::Line {
        scale,
        offset: low.to - scale * low.from,
      }
    };

    pairs
      .iter()
      .all(|pair| (remap.apply(pair.from) - pair.to).abs() <= Self::REMAP_TOLERANCE)
      .then_some(remap)
  }
}

/// One vertex's coordinate on one axis: `from` the source's, `to` the level's.
struct LevelDetailRemapPair {
  from: f32,
  to: f32,
}

/// How one axis of a source's coordinates moves into the atlas.
#[derive(Clone, Copy)]
enum LevelDetailAxisRemap {
  /// `scale * from + offset`, fitted where the source spans the axis.
  Line { scale: f32, offset: f32 },
  /// `from + offset`, where the source spans nothing on the axis, so no scale can be told.
  Shift { offset: f32 },
}

impl LevelDetailAxisRemap {
  fn apply(self, from: f32) -> f32 {
    match self {
      Self::Line { scale, offset } => scale * from + offset,
      Self::Shift { offset } => from + offset,
    }
  }
}
