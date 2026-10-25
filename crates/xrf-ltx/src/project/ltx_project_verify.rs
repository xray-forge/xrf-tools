use std::path::Path;
use std::sync::Arc;

use fxhash::FxBuildHasher;
use indexmap::IndexSet;
use xrf_error::XrfResult;
use xrf_job::{JobOutcome, JobScope};
use xrf_output::OutputOptions;
use xrf_utils::format_path;
use xrf_vfs::XrayLogicalPath;

use crate::dialect::LtxResolution;
use crate::finding::LtxFinding;
use crate::ltx::Ltx;
use crate::project::{LTX_PHASE_VERIFY, LtxEntryVerification, LtxProject, LtxProjectVerifyResult, LtxVerifyOptions};
use crate::syntax::{LTX_SCHEME_FIELD, LTX_SYMBOL_ANY};

impl LtxProject {
  /// Verify all the entries in current ltx project.
  /// Make sure that:
  /// - All included files exist or `.ts` counterpart is declared
  /// - All the inherited sections are valid and declared before inherit attempt
  pub fn verify_entries_opt(&self, options: LtxVerifyOptions) -> XrfResult<LtxProjectVerifyResult> {
    let mut result: LtxProjectVerifyResult = LtxProjectVerifyResult::new();

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

      // One unreadable config must not end the run.
      // Not retained: a sweep reads every root once, and holding them all is what `is_caching_resolutions` exists to
      // let a session opt into rather than something every verify pays for.
      let resolved: Arc<LtxResolution> = match self.read_resolution(entry) {
        Ok(resolved) => resolved,
        Err(error) => {
          result
            .findings
            .push(LtxFinding::new_unreadable(entry.as_str(), error.to_string()));

          continue;
        }
      };

      result.absorb(self.verify_resolved_opt(entry, &resolved, Some(&options.output)));
    }

    result.duration = options.job.elapsed();

    for finding in &result.findings {
      xrf_output::error!(options.output, "{}", self.with_reported_paths(finding));
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

    xrf_output::info!(options.output, "Found {} finding(s)", result.findings.len());

    Ok(result)
  }

  /// Verifies one entry point that the caller has already resolved.
  ///
  /// The door for a surface inspecting a single config: it holds a resolution for its own reasons and must not pay for
  /// a second one, and it wants the findings of that root alone rather than of the tree around it. The sweep above is
  /// this same check in a loop, so the two can never disagree about a file.
  pub fn verify_resolved(&self, entry: &XrayLogicalPath, resolution: &LtxResolution) -> LtxEntryVerification {
    self.verify_resolved_opt(entry, resolution, None)
  }

  /// The same finding with its entry point and file named by the path a person can act on: the file on disk when it
  /// is loose, the logical path when it is archived.
  pub fn with_reported_paths(&self, finding: &LtxFinding) -> LtxFinding {
    LtxFinding {
      entry: self.report_path(&finding.entry),
      file: finding.file.as_deref().map(|file| self.report_path(file)),
      ..finding.clone()
    }
  }

  /// The section-and-field check itself over one resolved config, and what its dialect said while resolving it.
  fn verify_resolved_opt(
    &self,
    entry: &XrayLogicalPath,
    resolution: &LtxResolution,
    output: Option<&OutputOptions>,
  ) -> LtxEntryVerification {
    let ltx: &Ltx = &resolution.ltx;
    let mut found: LtxEntryVerification = LtxEntryVerification::default();
    // Rendered once per entry point and only when someone reads it: rendering asks the VFS where the file sits.
    let reported: Option<String> = output.map(|_| self.report_path(entry.as_str()));

    for (section_name, section) in ltx.iter() {
      found.total_sections += 1;

      // The file a person has to open, which the entry point stays beside.
      let declared_in: Option<&str> = section.get_origin();

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
            if let (Some(output), Some(reported)) = (output, &reported) {
              xrf_output::verbose!(output, "Checking {} [{}] {}", reported, section_name, field_name);
            }

            found.checked_fields += 1;

            if let Err(message) = field_definition.validate_value(ltx, value) {
              section_has_error = true;

              found.findings.push(LtxFinding::new_scheme(
                entry.as_str(),
                declared_in.map(String::from),
                section_name,
                Some(field_name),
                message,
              ));
            }
          } else if scheme_definition.is_strict {
            section_has_error = true;

            found.findings.push(LtxFinding::new_scheme(
              entry.as_str(),
              declared_in.map(String::from),
              section_name,
              Some(field_name),
              "Unexpected field, definition is required in strict mode",
            ));
          }
        }

        if scheme_definition.is_strict {
          for (field_name, definition) in &scheme_definition.fields {
            if !definition.is_optional && field_name != LTX_SYMBOL_ANY && !validated.contains(field_name) {
              section_has_error = true;

              found.findings.push(LtxFinding::new_scheme(
                entry.as_str(),
                declared_in.map(String::from),
                section_name,
                Some(field_name),
                "Required field was not provided",
              ));
            }
          }
        }
      } else {
        section_has_error = true;

        found.findings.push(LtxFinding::new_scheme(
          entry.as_str(),
          declared_in.map(String::from),
          section_name,
          None,
          format!("Required schema '{scheme_name}' definition is not found"),
        ));
      }

      if section_has_error {
        found.invalid_sections += 1;
      } else {
        found.valid_sections += 1;
      }
    }

    // What the dialect noticed is judged with the rest: each is a statement the game loads and then ignores, which a
    // modder cannot see any other way.
    for diagnostic in &resolution.diagnostics {
      let declared_in: Option<&str> = ltx
        .section(&diagnostic.section)
        .and_then(|section| section.get_origin());

      found
        .findings
        .push(LtxFinding::from_diagnostic(entry.as_str(), diagnostic, declared_in));
    }

    found
  }

  /// One logical path as a person can act on it, or as written where the VFS will not accept it.
  fn report_path(&self, logical: &str) -> String {
    match XrayLogicalPath::new(logical) {
      Ok(logical) => format_path(&self.path_of(&logical)).to_string(),
      Err(_) => String::from(logical),
    }
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
