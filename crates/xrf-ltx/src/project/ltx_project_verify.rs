use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use fxhash::FxBuildHasher;
use indexmap::IndexSet;
use xrf_error::{XrfError, XrfResult};
use xrf_job::{JobOutcome, JobScope};
use xrf_output::OutputOptions;
use xrf_utils::format_path;
use xrf_vfs::XrayLogicalPath;

use crate::ltx::Ltx;
use crate::project::{LTX_PHASE_VERIFY, LtxEntryVerification, LtxProject, LtxProjectVerifyResult, LtxVerifyOptions};
use crate::syntax::{LTX_SCHEME_FIELD, LTX_SYMBOL_ANY};

/// Where a declaring file's logical path is rendered once and reused.
///
/// Rendering asks the VFS where a logical path really sits, and a config commonly declares hundreds of sections, so
/// this is paid per file rather than per finding.
type DeclaringPaths = HashMap<String, String>;

impl LtxProject {
  /// Verify all the entries in current ltx project.
  /// Make sure that:
  /// - All included files exist or `.ts` counterpart is declared
  /// - All the inherited sections are valid and declared before inherit attempt
  pub fn verify_entries_opt(&self, options: LtxVerifyOptions) -> XrfResult<LtxProjectVerifyResult> {
    let mut result: LtxProjectVerifyResult = LtxProjectVerifyResult::new();
    let mut declaring_paths: DeclaringPaths = HashMap::new();

    xrf_output::heading!(options.output, "Verify path: {}", format_path(&self.root));

    // Captured where the per-file work begins, so what came before it - mounting, indexing, assembling this project -
    // is named rather than lost.
    result.startup_duration = options.job.elapsed();

    let verifying: JobScope = options
      .job
      .enter(LTX_PHASE_VERIFY, Some(self.ltx_file_entries.len() as u64));

    // For each file entry in the project:
    for entry in &self.ltx_file_entries {
      // A stopped verification reports the findings it had reached and says it stopped. Without the outcome those
      // findings read as a complete verdict, which is the one way a partial check can do harm.
      if options.job.is_cancelled() {
        result.outcome = JobOutcome::Cancelled;

        break;
      }

      verifying.advance();

      // Do not check scheme definitions for scheme files - makes no sense.
      if Self::is_ltx_scheme_path(entry) {
        continue;
      } else {
        result.total_files += 1;
      }

      // Reported by the path a person can act on: the file on disk when it is loose, the logical path when it is archived.
      // Rendered once per file because a scheme error carries its location as a string and one file can raise many.
      let reported: String = format_path(&self.path_of(entry)).to_string();

      // One unreadable config must not end the run.
      let ltx: Arc<Ltx> = match self.read_full(entry) {
        Ok(ltx) => ltx,
        Err(error) => {
          result
            .errors
            .push(XrfError::new_verify_error(format!("Cannot read {reported}: {error}")));

          continue;
        }
      };

      result.absorb(self.verify_resolved_opt(&ltx, &reported, &mut declaring_paths, Some(&options.output))?);
    }

    result.duration = options.job.elapsed();

    for error in &result.errors {
      xrf_output::error!(options.output, "{error}");
    }

    xrf_output::info!(
      options.output,
      "Checked {} files, {} sections in {}",
      self.ltx_files.len(),
      result.total_sections,
      xrf_utils::format_duration(result.duration)
    );

    if result.total_sections == 0 {
      xrf_output::info!(options.output, "No LTX sections were verified");
    } else {
      xrf_output::info!(
        options.output,
        "Verified {:.2}%, {} files, {} sections, {} fields",
        (result.checked_sections as f32 * 100.0) / result.total_sections as f32,
        result.total_files,
        result.checked_sections,
        result.checked_fields
      );
    }

    xrf_output::info!(options.output, "Found {} error(s)", result.errors.len());

    Ok(result)
  }

  /// Verifies one entry point that the caller has already resolved.
  ///
  /// The door for a surface inspecting a single config: it holds a resolution for its own reasons and must not pay for
  /// a second one, and it wants the findings of that root alone rather than of the tree around it. The sweep above is
  /// this same check in a loop, so the two can never disagree about a file.
  ///
  /// `entry` names the entry point the resolution came from, and is what a finding reports as the file to re-run.
  ///
  /// # Errors
  ///
  /// Returns an error only for a scheme failure that is not a finding about the config, which is a defect in a scheme
  /// declaration rather than in the config being judged.
  pub fn verify_resolved(&self, entry: &XrayLogicalPath, ltx: &Ltx) -> XrfResult<LtxEntryVerification> {
    let reported: String = format_path(&self.path_of(entry)).to_string();

    self.verify_resolved_opt(ltx, &reported, &mut DeclaringPaths::new(), None)
  }

  /// The section-and-field check itself, over one resolved config.
  fn verify_resolved_opt(
    &self,
    ltx: &Ltx,
    reported: &str,
    declaring_paths: &mut DeclaringPaths,
    output: Option<&OutputOptions>,
  ) -> XrfResult<LtxEntryVerification> {
    let mut found: LtxEntryVerification = LtxEntryVerification::default();

    for (section_name, section) in ltx.iter() {
      found.total_sections += 1;

      // The file a person has to open. `reported` stays beside it, because the entry point is what a caller re-runs
      // and the only thing that explains why two files were read together.
      let declared_in: Option<String> = section
        .get_origin()
        .map(|origin| self.declaring_path_of(origin, declaring_paths));

      // Check only if schema is defined:
      let Some(scheme_name) = section.get(LTX_SCHEME_FIELD) else {
        found.skipped_sections += 1;

        continue;
      };

      let mut section_has_error: bool = false;

      found.checked_sections += 1;

      // Check if definition or required schema exists:
      if let Some(scheme_definition) = self.ltx_scheme_declarations.get(scheme_name) {
        let mut validated: IndexSet<String, FxBuildHasher> = Default::default();

        // Check all fields in section data.
        for (field_name, value) in section {
          validated.insert(field_name.into());

          // Respect `*` definition for mapping sections.
          if let Some(field_definition) = scheme_definition
            .fields
            .get(field_name)
            .or_else(|| scheme_definition.fields.get(LTX_SYMBOL_ANY))
          {
            if let Some(output) = output {
              xrf_output::verbose!(output, "Checking {} [{}] {}", reported, section_name, field_name);
            }

            found.checked_fields += 1;

            if let Some(error) = field_definition.validate_value(ltx, value) {
              match error {
                XrfError::LtxScheme { message, .. } => {
                  section_has_error = true;

                  found.errors.push(XrfError::new_scheme_error_resolved(
                    section_name,
                    field_name,
                    message,
                    declared_in.as_deref(),
                    reported,
                  ));
                }
                error => return Err(error),
              }
            }
          } else if scheme_definition.is_strict {
            section_has_error = true;

            found.errors.push(XrfError::new_scheme_error_resolved(
              section_name,
              field_name,
              "Unexpected field, definition is required in strict mode",
              declared_in.as_deref(),
              reported,
            ));
          }
        }

        if scheme_definition.is_strict {
          for (field_name, definition) in &scheme_definition.fields {
            if !definition.is_optional && field_name != LTX_SYMBOL_ANY && !validated.contains(field_name) {
              section_has_error = true;

              found.errors.push(XrfError::new_scheme_error_resolved(
                section_name,
                field_name,
                "Required field was not provided",
                declared_in.as_deref(),
                reported,
              ));
            }
          }
        }
      } else {
        section_has_error = true;

        found.errors.push(XrfError::new_scheme_error_resolved(
          section_name,
          "*",
          format!("Required schema '{scheme_name}' definition is not found"),
          declared_in.as_deref(),
          reported,
        ));
      }

      if section_has_error {
        found.invalid_sections += 1;
      } else {
        found.valid_sections += 1;
      }
    }

    Ok(found)
  }

  /// Renders one declaring file's logical path the way a person can act on it, once per file.
  fn declaring_path_of(&self, origin: &str, declaring_paths: &mut DeclaringPaths) -> String {
    declaring_paths
      .entry(String::from(origin))
      .or_insert_with(|| match XrayLogicalPath::new(origin) {
        Ok(logical) => format_path(&self.path_of(&logical)).to_string(),
        // A path the VFS will not accept is still worth naming as the file said it.
        Err(_) => String::from(origin),
      })
      .clone()
  }

  /// Verify all the section/field entries in current ltx project.
  pub fn verify_entries(&self) -> XrfResult<LtxProjectVerifyResult> {
    self.verify_entries_opt(Default::default())
  }

  /// Format single LTX file by provided path
  pub fn verify_file<P: AsRef<Path>>(path: P) -> XrfResult<()> {
    Ltx::read_from_file_standard(path)?;

    Ok(())
  }
}
