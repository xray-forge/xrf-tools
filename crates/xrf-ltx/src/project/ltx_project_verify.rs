use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use fxhash::FxBuildHasher;
use indexmap::IndexSet;
use xrf_error::{XrfError, XrfResult};
use xrf_job::{JobOutcome, JobScope};
use xrf_utils::format_path;
use xrf_vfs::XrayLogicalPath;

use crate::ltx::Ltx;
use crate::project::{LTX_PHASE_VERIFY, LtxProject, LtxProjectVerifyResult, LtxVerifyOptions};
use crate::syntax::{LTX_SCHEME_FIELD, LTX_SYMBOL_ANY};

impl LtxProject {
  /// Verify all the entries in current ltx project.
  /// Make sure that:
  /// - All included files exist or `.ts` counterpart is declared
  /// - All the inherited sections are valid and declared before inherit attempt
  pub fn verify_entries_opt(&self, options: LtxVerifyOptions) -> XrfResult<LtxProjectVerifyResult> {
    let mut result: LtxProjectVerifyResult = LtxProjectVerifyResult::new();
    // Rendered once per declaring file, not once per section: a config commonly declares hundreds of them, and
    // rendering asks the VFS where a logical path really sits.
    let mut declaring_paths: HashMap<String, String> = HashMap::new();

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

      // For each section in file:
      for (section_name, section) in ltx.iter() {
        result.total_sections += 1;

        // The file a person has to open. `reported` stays beside it, because the entry point is what a caller re-runs
        // and the only thing that explains why two files were read together.
        let declared_in: Option<String> = section.get_origin().map(|origin| {
          declaring_paths
            .entry(String::from(origin))
            .or_insert_with(|| match XrayLogicalPath::new(origin) {
              Ok(logical) => format_path(&self.path_of(&logical)).to_string(),
              // A path the VFS will not accept is still worth naming as the file said it.
              Err(_) => String::from(origin),
            })
            .clone()
        });

        // Check only if schema is defined:
        if let Some(scheme_name) = section.get(LTX_SCHEME_FIELD) {
          let mut section_has_error: bool = false;

          result.checked_sections += 1;

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
                xrf_output::verbose!(
                  options.output,
                  "Checking {} [{}] {}",
                  reported,
                  section_name,
                  field_name
                );

                result.checked_fields += 1;

                if let Some(error) = field_definition.validate_value(&ltx, value) {
                  match error {
                    XrfError::LtxScheme { message, .. } => {
                      section_has_error = true;

                      result.errors.push(XrfError::new_scheme_error_resolved(
                        section_name,
                        field_name,
                        message,
                        declared_in.as_deref(),
                        &reported,
                      ));
                    }
                    error => return Err(error),
                  }
                }
              } else if scheme_definition.is_strict {
                section_has_error = true;

                result.errors.push(XrfError::new_scheme_error_resolved(
                  section_name,
                  field_name,
                  "Unexpected field, definition is required in strict mode",
                  declared_in.as_deref(),
                  &reported,
                ));
              }
            }

            if scheme_definition.is_strict {
              for (field_name, definition) in &scheme_definition.fields {
                if !definition.is_optional && field_name != LTX_SYMBOL_ANY && !validated.contains(field_name) {
                  section_has_error = true;

                  result.errors.push(XrfError::new_scheme_error_resolved(
                    section_name,
                    field_name,
                    "Required field was not provided",
                    declared_in.as_deref(),
                    &reported,
                  ));
                }
              }
            }
          } else {
            section_has_error = true;

            result.errors.push(XrfError::new_scheme_error_resolved(
              section_name,
              "*",
              format!("Required schema '{scheme_name}' definition is not found"),
              declared_in.as_deref(),
              &reported,
            ));
          }

          if section_has_error {
            result.invalid_sections += 1;
          } else {
            result.valid_sections += 1;
          }
        } else {
          result.skipped_sections += 1
        }
      }
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

#[cfg(test)]
mod tests {
  use std::fs;
  use std::path::PathBuf;

  use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

  use super::*;
  use crate::project::{LtxProjectOptions, LtxVerifyOptions};

  #[test]
  fn the_total_covers_the_work_done_before_the_first_file_was_read() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/tests/ltx_project_verify/condlist");

    // Created before the project opens, exactly as the desktop command and the command line both do.
    let job: xrf_job::JobHandle = xrf_job::JobHandle::inert();

    let project = LtxProject::open_at_path_opt(
      &root,
      LtxProjectOptions {
        is_with_schemes_check: true,
        ..Default::default()
      },
    )
    .expect("Expected test project to open");

    let opened_at: std::time::Duration = job.elapsed();

    let result = project
      .verify_entries_opt(LtxVerifyOptions {
        job: job.clone(),
        ..Default::default()
      })
      .expect("Expected test project verification to complete");

    // What the old shape could not say: opening the project is inside the total.
    assert!(
      result.startup_duration >= opened_at,
      "startup ({:?}) has to contain the project opening already measured ({opened_at:?})",
      result.startup_duration
    );

    // And the total contains the startup, so the per-file phase is the difference rather than the whole answer.
    assert!(
      result.duration >= result.startup_duration,
      "total ({:?}) has to contain startup ({:?})",
      result.duration,
      result.startup_duration
    );
  }

  /// A handle made at the call site measures only what it wrapped, which is the honest reading for a caller that did
  /// not want its own setup counted - `xrf-gamedata` verifying LTX as one of its checks, for instance.
  #[test]
  fn a_handle_made_at_the_call_site_reports_almost_no_startup() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/tests/ltx_project_verify/condlist");
    let project = LtxProject::open_at_path_opt(
      &root,
      LtxProjectOptions {
        is_with_schemes_check: true,
        ..Default::default()
      },
    )
    .expect("Expected test project to open");

    let result = project
      .verify_entries_opt(LtxVerifyOptions { ..Default::default() })
      .expect("Expected test project verification to complete");

    assert!(result.duration >= result.startup_duration);
  }

  #[test]
  fn validates_condlists_from_project_schemes() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/tests/ltx_project_verify/condlist");
    let project = LtxProject::open_at_path_opt(
      &root,
      LtxProjectOptions {
        is_with_schemes_check: true,
        ..Default::default()
      },
    )
    .expect("Expected test project to open");

    let result = project
      .verify_entries_opt(LtxVerifyOptions { ..Default::default() })
      .expect("Expected test project verification to complete");

    assert_eq!(result.valid_sections, 2);
    assert_eq!(result.invalid_sections, 1);
    assert_eq!(result.errors.len(), 1);
    assert_eq!(
      result.errors[0].to_string(),
      format!(
        "Ltx scheme error in '{}' [invalid] value : Parsing error: Invalid condlist syntax at byte 2: Expected a name after condition or effect prefix",
        root.join("invalid.ltx").display(),
      ),
    );
  }

  #[test]
  fn skips_schema_less_sections() -> XrfResult {
    let root: PathBuf = build_absolute_generated_test_resource_path("project_verify/schema_less");

    let _ = fs::remove_dir_all(&root);

    fs::create_dir_all(&root)?;
    fs::write(root.join("array_sections.ltx"), "[array@one]\nvalue = 1\n")?;

    let project: LtxProject = LtxProject::open_at_path_opt(
      &root,
      LtxProjectOptions {
        is_with_schemes_check: true,
        ..Default::default()
      },
    )?;
    let result: LtxProjectVerifyResult = project.verify_entries_opt(LtxVerifyOptions { ..Default::default() })?;

    assert_eq!(result.checked_sections, 0);
    assert_eq!(result.skipped_sections, 1);
    assert_eq!(result.invalid_sections, 0);
    assert!(result.errors.is_empty());

    fs::remove_dir_all(root)?;

    Ok(())
  }

  #[test]
  fn skips_inheritance_for_entry_with_header_metadata() -> XrfResult {
    let root: PathBuf = build_absolute_generated_test_resource_path("project_verify/skip_inheritance");

    let _ = fs::remove_dir_all(&root);

    fs::create_dir_all(&root)?;
    fs::write(
      root.join("disabled.ltx"),
      "; @xrf-ltx skip-inheritance\n[child]:missing\n",
    )?;

    let project: LtxProject = LtxProject::open_at_path(&root)?;
    let result: LtxProjectVerifyResult = project.verify_entries()?;

    assert_eq!(result.total_files, 1);
    assert_eq!(result.total_sections, 1);
    assert!(result.errors.is_empty());

    fs::remove_dir_all(root)?;

    Ok(())
  }

  #[test]
  fn reports_an_unreadable_entry_and_keeps_verifying_the_rest() -> XrfResult {
    // A real installation holds orphan configs that inherit sections only the rest of the tree defines. Ending the run at
    // the first one would leave every later config unverified.
    let root: PathBuf = build_absolute_generated_test_resource_path("project_verify/unreadable");

    let _ = fs::remove_dir_all(&root);

    fs::create_dir_all(&root)?;
    fs::write(root.join("broken.ltx"), "[child]:missing\n")?;
    fs::write(root.join("valid.ltx"), "[section]\nvalue = 1\n")?;

    let project: LtxProject = LtxProject::open_at_path(&root)?;
    let result: LtxProjectVerifyResult = project.verify_entries()?;

    assert_eq!(result.total_files, 2);
    assert_eq!(result.errors.len(), 1, "one finding rather than a failed run");
    assert!(
      result.errors[0].to_string().contains("broken.ltx"),
      "the finding names the file to act on"
    );
    assert_eq!(result.total_sections, 1, "the readable entry is still verified");

    fs::remove_dir_all(root)?;

    Ok(())
  }
}
