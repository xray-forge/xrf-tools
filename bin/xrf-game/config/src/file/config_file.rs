use std::sync::Arc;

use xrf_error::XrfResult;
use xrf_ltx::{Ltx, LtxResolution, Section};

use crate::issue::config_issue::ConfigIssue;
use crate::issue::config_issues::ConfigIssues;
use crate::read::config_reader::ConfigReader;

/// One config as the game reads it, resolved in its tree's dialect (includes, inheritance, a mod's overrides) or read
/// as plain text, such as an object's custom data. Shared, never copied: a resolution a project keeps is held as it is.
#[derive(Clone, Debug)]
pub struct ConfigFile {
  /// Its path under `configs`, or what the text came from, as its issues name it.
  path: String,
  resolution: Arc<LtxResolution>,
}

impl ConfigFile {
  /// The config at `path` as `resolution` resolved it.
  pub fn of_resolution(path: &str, resolution: Arc<LtxResolution>) -> Self {
    Self {
      path: String::from(path),
      resolution,
    }
  }

  /// `text` read as a config named `path`, nothing resolved.
  ///
  /// # Errors
  ///
  /// Returns why `text` is no LTX.
  pub fn parse(path: &str, text: &str) -> XrfResult<Self> {
    Ok(Self::of_resolution(
      path,
      Arc::new(LtxResolution::new_plain(Ltx::read_from_str(text)?)),
    ))
  }

  pub fn get_path(&self) -> &str {
    &self.path
  }

  pub fn has_section(&self, name: &str) -> bool {
    self.get_ltx().section(name).is_some()
  }

  /// The names of its sections, in the file's order.
  pub fn get_section_names(&self) -> impl Iterator<Item = &str> {
    self.get_ltx().iter().map(|(name, _)| name)
  }

  /// Reads the section `name`, collecting into `issues`; none where the file lacks it, which is no issue.
  pub fn read_section<'f, 'i>(&'f self, name: &'f str, issues: &'i mut ConfigIssues) -> Option<ConfigReader<'f, 'i>> {
    let section: &Section = self.get_ltx().section(name)?;

    Some(ConfigReader::new(&self.path, name, section, issues))
  }

  /// Reads a section the game needs: none, said as an issue, where the file lacks it.
  pub fn require_section<'f, 'i>(
    &'f self,
    name: &'f str,
    issues: &'i mut ConfigIssues,
  ) -> Option<ConfigReader<'f, 'i>> {
    if !self.has_section(name) {
      issues.push(ConfigIssue {
        file: self.path.clone(),
        section: Some(String::from(name)),
        key: None,
        message: String::from("is missing"),
      });

      return None;
    }

    self.read_section(name, issues)
  }

  fn get_ltx(&self) -> &Ltx {
    &self.resolution.ltx
  }
}
