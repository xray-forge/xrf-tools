//! Packs every visual under a path and accounts for what came out.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use xrf_error::XrfError;
use xrf_extension::XrayExtension;
use xrf_math::Vector3d;
use xrf_ogf::OgfFile;
use xrf_report::{CheckId, CheckReport, Finding, Report, RuleId, Status};
use xrf_spawn::XRayByteOrder;
use xrf_utils::{format_path, to_portable_path_string};
use xrf_visual::{VisualBounds, VisualDescription, VisualPackage, VisualPacker, VisualSkipCause, VisualSubmesh};

use crate::commands::ogf::verify::ogf_texture_resolver::OgfTextureResolver;
use crate::commands::ogf::verify::ogf_verification_census::OgfVerificationCensus;
use crate::commands::ogf::verify::ogf_verification_result::OgfVerificationResult;
use crate::commands::ogf::verify::texture_resolution::TextureResolution;
use crate::core::host_walk::HostWalk;

/// Fraction of a declared bounding box's diagonal that geometry may exceed it by before it is worth
/// reporting. Exporters pad declared bounds, and float error is unavoidable, so only a gross
/// disagreement says anything.
const BOUNDS_TOLERANCE_RATIO: f32 = 0.01;
const BOUNDS_TOLERANCE_FLOOR: f32 = 0.001;

pub struct OgfVerifier<'a> {
  root: &'a Path,
  /// Roots layered behind each visual's own tree, so an overlay is measured the way the engine reads it.
  roots: Vec<PathBuf>,
}

impl<'a> OgfVerifier<'a> {
  /// How far a vertex's weights may sum from one before it is counted as astray, which is float noise rather than a
  /// judgement about the file.
  const SKIN_WEIGHT_TOLERANCE: f32 = 1e-3;

  pub fn new(root: &'a Path, roots: Vec<PathBuf>) -> Self {
    Self { root, roots }
  }

  pub fn run(&self) -> OgfVerificationResult {
    let started_at: Instant = Instant::now();

    let mut census: OgfVerificationCensus = OgfVerificationCensus::default();
    let mut read_findings: Vec<Finding> = Vec::new();
    let mut geometry_findings: Vec<Finding> = Vec::new();
    let mut bounds_findings: Vec<Finding> = Vec::new();
    let mut texture_findings: Vec<Finding> = Vec::new();
    let mut textures: OgfTextureResolver = OgfTextureResolver::new(self.roots.clone());

    // Files the sweep could not read at all, as opposed to files it read and judged malformed.
    let mut io_failures: usize = 0;

    for path in HostWalk::files_with(self.root, XrayExtension::Ogf) {
      // A directory the sweep could not walk is a read finding: its visuals were never judged, which is no pass.
      let path: PathBuf = match path {
        Ok(path) => path,
        Err(error) => {
          census.unreadable_files += 1;
          io_failures += 1;
          read_findings.push(Finding::new(RuleId::of_static("ogf.read"), None, error.to_string()));

          continue;
        }
      };

      census.files += 1;

      // A read error is a value here, but a panic inside the reader is not: the release profile aborts,
      // which is the right outcome because a panic is a defect rather than a property of the file.
      let file: OgfFile = match OgfFile::read_from_path::<XRayByteOrder, _>(&path) {
        Ok(file) => file,
        Err(error) => {
          census.unreadable_files += 1;

          if matches!(error, XrfError::Io { .. }) {
            io_failures += 1;
          }

          read_findings.push(Finding::new(
            RuleId::of_static("ogf.read"),
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

    // A rejected file is invalid input, which `Status::from_is_valid` grades as failed. A file or directory that could
    // not be read at all is the run's own failure: `Status::Error` outranks every verdict, so the sweep answers an
    // execution failure rather than a judgement on visuals it never saw.
    let read_status: Status = if io_failures > 0 {
      Status::Error
    } else {
      Status::from_is_valid(read_findings.is_empty())
    };
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

    // Nothing swept is not a pass: a path holding no visual judged nothing, which every check says.
    let [read_status, geometry_status, bounds_status, texture_status]: [Status; 4] =
      if census.files == 0 && read_findings.is_empty() {
        [Status::Skipped; 4]
      } else {
        [read_status, geometry_status, bounds_status, texture_status]
      };

    let duration: Duration = started_at.elapsed();

    OgfVerificationResult {
      report: Report::new(vec![
        CheckReport::new(
          CheckId::of_static("ogf.read"),
          read_status,
          Some(duration),
          read_findings,
        ),
        CheckReport::new(
          CheckId::of_static("ogf.geometry"),
          geometry_status,
          Some(duration),
          geometry_findings,
        ),
        CheckReport::new(
          CheckId::of_static("ogf.bounds"),
          bounds_status,
          Some(duration),
          bounds_findings,
        ),
        CheckReport::new(
          CheckId::of_static("ogf.textures"),
          texture_status,
          Some(duration),
          texture_findings,
        ),
      ]),
      census,
      duration,
    }
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
  fn census_skin(census: &mut OgfVerificationCensus, package: &VisualPackage) {
    for submesh in &package.description.submeshes {
      let Some(skin) = submesh.geometry().and_then(|it| it.skin.as_ref()) else {
        continue;
      };

      census.skinned_submeshes += 1;

      let start: usize = skin.weights.byte_offset as usize;
      let end: usize = start + skin.weights.byte_length as usize;
      let weights: Vec<f32> = package.buffer[start..end]
        .as_chunks::<{ size_of::<f32>() }>()
        .0
        .iter()
        .map(|bytes| f32::from_le_bytes(*bytes))
        .collect();

      census.vertices_with_stray_skin_weights += weights
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|vertex| (vertex.iter().sum::<f32>() - 1.0).abs() > Self::SKIN_WEIGHT_TOLERANCE)
        .count();
    }
  }

  /// Resolve every submesh's texture reference, counting formats and reporting what did not resolve.
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
            RuleId::of_static("ogf.textures.root_missing"),
            Some(subject),
            format!("No directory above the visual holds both meshes and textures, so '{reference}' cannot resolve"),
          ));
        }
        TextureResolution::Missing { sources } => {
          census.missing_texture_references += 1;
          findings.push(Finding::new(
            RuleId::of_static("ogf.textures.missing"),
            Some(subject),
            format!("'{reference}' does not resolve under {}", sources.join(", ")),
          ));
        }
        // Counted as resolved, and left out of the format census: it is present, and what it is stays unread.
        TextureResolution::Located { step, .. } => {
          census.resolved_texture_references += 1;
          census.located_texture_references += 1;
          OgfVerificationCensus::count(&mut census.texture_steps, step);
        }
        // Counted as missing, which leaves the check incomplete: what the root holds was never looked at.
        TextureResolution::RootUnreadable { root, reason } => {
          census.missing_texture_references += 1;
          findings.push(Finding::new(
            RuleId::of_static("ogf.textures.root_unreadable"),
            Some(subject),
            format!(
              "'{reference}' was not looked for, {} could not be searched: {reason}",
              format_path(&root)
            ),
          ));
        }
        TextureResolution::Rejected { reason } => {
          census.missing_texture_references += 1;
          findings.push(Finding::new(
            RuleId::of_static("ogf.textures.rejected"),
            Some(subject),
            format!("'{reference}' names no path the engine resolves: {reason}"),
          ));
        }
        TextureResolution::Unreadable { path: texture, reason } => {
          census.unreadable_textures += 1;
          findings.push(Finding::new(
            RuleId::of_static("ogf.textures.unreadable"),
            Some(subject),
            format!("'{}' would not parse: {reason}", format_path(&texture)),
          ));
        }
        TextureResolution::Resolved {
          step, format, metadata, ..
        } => {
          census.resolved_texture_references += 1;
          OgfVerificationCensus::count(&mut census.texture_steps, step);
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
        let rule: &'static str = match cause {
          VisualSkipCause::Unsupported => "ogf.geometry.unsupported",
          VisualSkipCause::Malformed => "ogf.geometry.malformed",
        };

        Some(Finding::new(
          RuleId::of_static(rule),
          Some(format!("{}#{}", self.subject(path), submesh.index)),
          reason,
        ))
      })
      .collect()
  }

  /// Report geometry that reaches outside the extent its header declares.
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
      RuleId::of_static("ogf.bounds.outside"),
      Some(self.subject(path)),
      format!(
        "Geometry reaches {excess} past the declared bounding box, tolerance {tolerance}. \
         Declared {} to {}, measured {} to {}",
        Self::describe_point(&declared.bounding_box.min),
        Self::describe_point(&declared.bounding_box.max),
        Self::describe_point(&computed.bounding_box.min),
        Self::describe_point(&computed.bounding_box.max)
      ),
    ))
  }

  /// A corner as `ogf info` prints one.
  fn describe_point(point: &Vector3d) -> String {
    format!("[{}, {}, {}]", point.x, point.y, point.z)
  }

  /// Subject of a finding: the path relative to the swept root, with forward slashes, matching how
  /// gamedata verification names assets.
  fn subject(&self, path: &Path) -> String {
    to_portable_path_string(path.strip_prefix(self.root).unwrap_or(path))
  }
}
