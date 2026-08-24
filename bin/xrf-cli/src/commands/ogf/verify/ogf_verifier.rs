//! Packs every visual under a path and accounts for what came out.
//!
//! Lives in the CLI rather than in `xrf-visual` so that crate stays a pure renderer projection with no
//! filesystem or reporting surface. A gamedata wide visual check belongs with the other gamedata
//! verifiers, not here.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use walkdir::WalkDir;
use xrf_db::{OgfFile, XRayByteOrder};
use xrf_report::{CheckId, CheckReport, Finding, Report, RuleId, Status};
use xrf_visual::{VisualBounds, VisualDescription, VisualPackage, VisualPacker, VisualSkipCause, VisualSubmesh};

use crate::commands::ogf::verify::ogf_texture_resolver::{OgfTextureResolver, TextureResolution};

const OGF_EXTENSION: &str = "ogf";

/// Fraction of a declared bounding box's diagonal that geometry may exceed it by before it is worth
/// reporting. Exporters pad declared bounds, and float error is unavoidable, so only a gross
/// disagreement says anything.
const BOUNDS_TOLERANCE_RATIO: f32 = 0.01;
const BOUNDS_TOLERANCE_FLOOR: f32 = 0.001;

/// What a sweep counted, beside what it found.
///
/// These are the numbers that decide whether the premises this work rests on hold: that loose visuals
/// are uniformly version 4 skeletons with skinned vertices, and that a progressive submesh's finest
/// detail level is usually not its whole index buffer.
#[derive(Debug, Default)]
pub struct OgfVerificationCensus {
  pub files: usize,
  pub unreadable_files: usize,
  pub files_without_geometry: usize,
  pub submeshes: usize,
  pub packed_submeshes: usize,
  pub unsupported_submeshes: usize,
  pub malformed_submeshes: usize,
  pub progressive_submeshes: usize,
  pub progressive_submeshes_drawing_part_of_the_buffer: usize,
  pub skinned_submeshes: usize,
  /// Vertices whose four weights do not sum to one, which a renderer would draw pulled toward the origin.
  pub vertices_with_stray_skin_weights: usize,
  pub bounds_disagreements: usize,
  pub versions: BTreeMap<u8, usize>,
  pub root_model_types: BTreeMap<String, usize>,
  pub submesh_model_types: BTreeMap<String, usize>,
  pub vertex_formats: BTreeMap<String, usize>,
  pub visuals_without_root: usize,
  pub texture_references: usize,
  pub resolved_texture_references: usize,
  pub missing_texture_references: usize,
  pub unreadable_textures: usize,
  pub distinct_textures: usize,
  pub textures_without_mipmaps: usize,
  pub texture_formats: BTreeMap<String, usize>,
  pub texture_sizes: BTreeMap<String, usize>,
}

impl OgfVerificationCensus {
  fn count(map: &mut BTreeMap<String, usize>, key: impl Into<String>) {
    *map.entry(key.into()).or_default() += 1;
  }
}

/// Outcome of a sweep: a finalized report and the counts behind it.
#[derive(Debug)]
pub struct OgfVerificationResult {
  pub census: OgfVerificationCensus,
  pub duration: Duration,
  pub report: Report,
}

pub struct OgfVerifier<'a> {
  root: &'a Path,
}

impl<'a> OgfVerifier<'a> {
  /// How far a vertex's weights may sum from one before it is counted as astray, which is float noise rather than a
  /// judgement about the file.
  const SKIN_WEIGHT_TOLERANCE: f32 = 1e-3;

  pub fn new(root: &'a Path) -> Self {
    Self { root }
  }

  pub fn run(&self) -> OgfVerificationResult {
    let started_at: Instant = Instant::now();

    let mut census: OgfVerificationCensus = OgfVerificationCensus::default();
    let mut read_findings: Vec<Finding> = Vec::new();
    let mut geometry_findings: Vec<Finding> = Vec::new();
    let mut bounds_findings: Vec<Finding> = Vec::new();
    let mut texture_findings: Vec<Finding> = Vec::new();
    let mut textures: OgfTextureResolver = OgfTextureResolver::default();

    for path in self.visual_paths() {
      census.files += 1;

      // A read error is a value here, but a panic inside the reader is not: the release profile aborts,
      // which is the right outcome because a panic is a defect rather than a property of the file.
      let file: OgfFile = match OgfFile::read_from_path::<XRayByteOrder, _>(&path) {
        Ok(file) => file,
        Err(error) => {
          census.unreadable_files += 1;
          read_findings.push(Finding::new(
            Self::rule("visuals.read"),
            Some(self.subject(&path)),
            error.to_string(),
          ));

          continue;
        }
      };

      self.census_source_formats(&mut census, &file);

      let package: VisualPackage = VisualPacker::pack(&file);

      self.census_package(&mut census, &package.description);
      Self::census_skin(&mut census, &package);

      geometry_findings.extend(self.geometry_findings(&path, &package.description));

      if let Some(finding) = self.bounds_finding(&path, &package.description) {
        census.bounds_disagreements += 1;
        bounds_findings.push(finding);
      }

      texture_findings.extend(self.texture_findings(&mut census, &mut textures, &path, &package.description));

      if package.description.submeshes.iter().all(|it| it.geometry().is_none()) {
        census.files_without_geometry += 1;
      }
    }

    // A rejected file is invalid input, which `Status::from_is_valid` grades as failed. `Status::Error`
    // is reserved for the checker itself breaking, and a reader that breaks panics rather than
    // returning, so the sweep aborts instead of reaching this.
    let read_status: Status = Status::from_is_valid(read_findings.is_empty());
    let geometry_status: Status = if census.malformed_submeshes > 0 {
      Status::Failed
    } else if census.unsupported_submeshes > 0 {
      Status::Incomplete
    } else {
      Status::Passed
    };
    let bounds_status: Status = match bounds_findings.is_empty() {
      true => Status::Passed,
      false => Status::Incomplete,
    };

    let texture_status: Status = if census.unreadable_textures > 0 {
      Status::Failed
    } else if census.missing_texture_references > 0 || census.visuals_without_root > 0 {
      // A reference the engine would answer with its dummy, or a visual outside any root. Both are properties of the
      // tree rather than of this tool, and both are the common case for an overlay tree.
      Status::Incomplete
    } else {
      Status::Passed
    };

    census.distinct_textures = textures.distinct_textures();

    let duration: Duration = started_at.elapsed();

    OgfVerificationResult {
      report: Report::new(vec![
        CheckReport::new(Self::check("read"), read_status, Some(duration), read_findings),
        CheckReport::new(
          Self::check("geometry"),
          geometry_status,
          Some(duration),
          geometry_findings,
        ),
        CheckReport::new(Self::check("bounds"), bounds_status, Some(duration), bounds_findings),
        CheckReport::new(
          Self::check("textures"),
          texture_status,
          Some(duration),
          texture_findings,
        ),
      ]),
      census,
      duration,
    }
  }

  /// Every visual the sweep covers: one named file, or every `.ogf` under a directory.
  fn visual_paths(&self) -> Vec<PathBuf> {
    if self.root.is_file() {
      return vec![self.root.to_path_buf()];
    }

    WalkDir::new(self.root)
      .into_iter()
      .filter_map(Result::ok)
      .filter(|entry| entry.file_type().is_file())
      .map(|entry| entry.into_path())
      .filter(|path| {
        path
          .extension()
          .is_some_and(|it| it.eq_ignore_ascii_case(OGF_EXTENSION))
      })
      .collect()
  }

  /// Count what the file declares, before packing has a chance to normalise any of it away.
  fn census_source_formats(&self, census: &mut OgfVerificationCensus, file: &OgfFile) {
    *census.versions.entry(file.header.version).or_default() += 1;

    let sources: Vec<&OgfFile> = match file.children.as_ref().map(|it| it.nested.as_slice()) {
      Some(nested) if !nested.is_empty() => nested.iter().collect(),
      _ => vec![file],
    };

    for source in sources {
      let format: String = match source.geometry.as_ref().and_then(|it| it.vertex_format) {
        Some(format) => format!("{format:#010x}"),
        None => String::from("none"),
      };

      OgfVerificationCensus::count(&mut census.vertex_formats, format);
    }
  }

  fn census_package(&self, census: &mut OgfVerificationCensus, description: &VisualDescription) {
    OgfVerificationCensus::count(&mut census.root_model_types, description.model_type_label.clone());

    for submesh in &description.submeshes {
      census.submeshes += 1;
      OgfVerificationCensus::count(&mut census.submesh_model_types, submesh.model_type_label.clone());

      match submesh.geometry() {
        Some(geometry) => {
          census.packed_submeshes += 1;

          if submesh.is_progressive() {
            census.progressive_submeshes += 1;

            if geometry.get_default_level().count < geometry.index_count {
              census.progressive_submeshes_drawing_part_of_the_buffer += 1;
            }
          }
        }
        None => match submesh.skipped().map(|(cause, _)| cause) {
          Some(VisualSkipCause::Unsupported) => census.unsupported_submeshes += 1,
          Some(VisualSkipCause::Malformed) => census.malformed_submeshes += 1,
          None => {}
        },
      }
    }
  }

  /// Count skinned submeshes, and the vertices among them whose weights do not sum to one.
  ///
  /// Read out of the packed buffer rather than off the parsed vertices, because the buffer is what a renderer skins
  /// with: the last weight of a set is reconstructed by the reader, so a sum that drifts is a statement about the
  /// arithmetic this crate performs and not only about the file. A vertex whose weights sum to less than one is drawn
  /// pulled toward the origin, which is why the count is worth having rather than assumed.
  fn census_skin(census: &mut OgfVerificationCensus, package: &VisualPackage) {
    for submesh in &package.description.submeshes {
      let Some(skin) = submesh.geometry().and_then(|it| it.skin.as_ref()) else {
        continue;
      };

      census.skinned_submeshes += 1;

      let start: usize = skin.weights.byte_offset as usize;
      let end: usize = start + skin.weights.byte_length as usize;
      let weights: Vec<f32> = package.buffer[start..end]
        .chunks_exact(size_of::<f32>())
        .map(|bytes| f32::from_le_bytes(bytes.try_into().expect("four bytes make one f32")))
        .collect();

      census.vertices_with_stray_skin_weights += weights
        .chunks_exact(4)
        .filter(|vertex| (vertex.iter().sum::<f32>() - 1.0).abs() > Self::SKIN_WEIGHT_TOLERANCE)
        .count();
    }
  }

  /// Resolve every submesh's texture reference, counting formats and reporting what did not resolve.
  ///
  /// A miss is not a defect: the engine substitutes its own dummy for exactly this case, so the interesting output is the
  /// distribution rather than a pass or fail. What would change the design is a format no renderer can upload.
  fn texture_findings(
    &self,
    census: &mut OgfVerificationCensus,
    textures: &mut OgfTextureResolver,
    path: &Path,
    description: &VisualDescription,
  ) -> Vec<Finding> {
    let mut findings: Vec<Finding> = Vec::new();

    for submesh in &description.submeshes {
      let Some(reference) = submesh.texture_name.as_deref() else {
        continue;
      };

      census.texture_references += 1;

      let subject: String = format!("{}#{}", self.subject(path), submesh.index);

      match textures.resolve(path, reference) {
        TextureResolution::NoRoot => {
          census.visuals_without_root += 1;
          findings.push(Finding::new(
            Self::rule("visuals.textures.root_missing"),
            Some(subject),
            format!("No directory above the visual holds both meshes and textures, so '{reference}' cannot resolve"),
          ));
        }
        TextureResolution::Missing { root } => {
          census.missing_texture_references += 1;
          findings.push(Finding::new(
            Self::rule("visuals.textures.missing"),
            Some(subject),
            format!("'{reference}' does not resolve under {}", root.display()),
          ));
        }
        TextureResolution::Unreadable { path: texture, reason } => {
          census.unreadable_textures += 1;
          findings.push(Finding::new(
            Self::rule("visuals.textures.unreadable"),
            Some(subject),
            format!("'{}' would not parse: {reason}", texture.display()),
          ));
        }
        TextureResolution::Resolved { format, metadata, .. } => {
          census.resolved_texture_references += 1;
          OgfVerificationCensus::count(&mut census.texture_formats, format);
          OgfVerificationCensus::count(
            &mut census.texture_sizes,
            format!("{}x{}", metadata.width, metadata.height),
          );

          if metadata.mipmap_levels <= 1 {
            census.textures_without_mipmaps += 1;
          }
        }
      }
    }

    findings
  }

  fn geometry_findings(&self, path: &Path, description: &VisualDescription) -> Vec<Finding> {
    description
      .submeshes
      .iter()
      .filter_map(|submesh: &VisualSubmesh| {
        let (cause, reason) = submesh.skipped()?;
        let rule: &str = match cause {
          VisualSkipCause::Unsupported => "visuals.geometry.unsupported",
          VisualSkipCause::Malformed => "visuals.geometry.malformed",
        };

        Some(Finding::new(
          Self::rule(rule),
          Some(format!("{}#{}", self.subject(path), submesh.index)),
          reason,
        ))
      })
      .collect()
  }

  /// Report geometry that reaches outside the extent its header declares.
  ///
  /// Only that direction matters: declared bounds are routinely padded, so geometry sitting well
  /// inside them says nothing, while geometry outside them means the engine would cull a model before
  /// it left the screen.
  fn bounds_finding(&self, path: &Path, description: &VisualDescription) -> Option<Finding> {
    let computed: &VisualBounds = description.computed_bounds.as_ref()?;
    let declared: &VisualBounds = &description.declared_bounds;

    let diagonal: f32 = (declared.bounding_box.max.x - declared.bounding_box.min.x).abs()
      + (declared.bounding_box.max.y - declared.bounding_box.min.y).abs()
      + (declared.bounding_box.max.z - declared.bounding_box.min.z).abs();
    let tolerance: f32 = (diagonal * BOUNDS_TOLERANCE_RATIO).max(BOUNDS_TOLERANCE_FLOOR);

    let excess: f32 = [
      declared.bounding_box.min.x - computed.bounding_box.min.x,
      declared.bounding_box.min.y - computed.bounding_box.min.y,
      declared.bounding_box.min.z - computed.bounding_box.min.z,
      computed.bounding_box.max.x - declared.bounding_box.max.x,
      computed.bounding_box.max.y - declared.bounding_box.max.y,
      computed.bounding_box.max.z - declared.bounding_box.max.z,
    ]
    .into_iter()
    .fold(0.0, f32::max);

    if excess <= tolerance {
      return None;
    }

    Some(Finding::new(
      Self::rule("visuals.bounds.outside"),
      Some(self.subject(path)),
      format!(
        "Geometry reaches {excess} past the declared bounding box, tolerance {tolerance}. \
         Declared {:?} to {:?}, measured {:?} to {:?}",
        declared.bounding_box.min, declared.bounding_box.max, computed.bounding_box.min, computed.bounding_box.max
      ),
    ))
  }

  /// Subject of a finding: the path relative to the swept root, with forward slashes, matching how
  /// gamedata verification names assets.
  fn subject(&self, path: &Path) -> String {
    path
      .strip_prefix(self.root)
      .unwrap_or(path)
      .to_string_lossy()
      .replace('\\', "/")
  }

  fn check(id: &str) -> CheckId {
    CheckId::new(id).expect("Expected a non-empty check id")
  }

  fn rule(id: &str) -> RuleId {
    RuleId::new(id).expect("Expected a non-empty rule id")
  }
}
