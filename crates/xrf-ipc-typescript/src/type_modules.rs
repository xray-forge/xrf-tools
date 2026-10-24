/// Which `types/` module a workspace crate's declarations land in: the crate's own, named after it (`xrf_renderer`
/// writes `xrf-renderer.ts`), unless the caller groups a family of crates into one.
///
/// A family keeps one surface one module to the frontend however its sources are split into crates, so moving code
/// between those crates moves no declaration between the frontend's imports.
#[derive(Clone, Debug, Default)]
pub struct TypeModules {
  /// Each family's crate name, and the module that crate and every `<crate name>_<part>` share.
  families: Vec<(&'static str, &'static str)>,
}

impl TypeModules {
  /// Puts the crate named `crate_name` and every crate named `<crate_name>_<part>` into the one module `module`.
  pub fn with_family(mut self, crate_name: &'static str, module: &'static str) -> Self {
    self.families.push((crate_name, module));
    self
  }

  /// The module owning what `module_path!()` places at `module_path`, read off its first segment, the declaring crate:
  /// its family's, else its own, `None` for a crate outside the workspace, which has no module.
  pub(crate) fn module_of(&self, module_path: &str) -> Option<String> {
    let declaring_crate: &str = module_path.split("::").next()?;

    for (crate_name, module) in &self.families {
      let is_member: bool = declaring_crate
        .strip_prefix(crate_name)
        .is_some_and(|rest| rest.is_empty() || rest.starts_with('_'));

      if is_member {
        return Some(module.to_string());
      }
    }

    declaring_crate
      .starts_with("xrf_")
      .then(|| declaring_crate.replace('_', "-"))
  }
}

#[cfg(test)]
mod tests {
  use super::TypeModules;

  #[test]
  fn a_crate_owns_a_module_named_after_it() {
    let modules: TypeModules = TypeModules::default();

    assert_eq!(
      modules.module_of("xrf_material::texture"),
      Some(String::from("xrf-material"))
    );
    assert_eq!(
      modules.module_of("xrf_app_levels::parts"),
      Some(String::from("xrf-app-levels"))
    );
    assert_eq!(modules.module_of("serde_json::value"), None);
  }

  #[test]
  fn a_family_of_crates_shares_its_module() {
    let modules: TypeModules = TypeModules::default().with_family("xrf_app", "xrf-app");

    for module_path in [
      "xrf_app::ipc",
      "xrf_app_core::session",
      "xrf_app_sprite_equipment::state",
    ] {
      assert_eq!(
        modules.module_of(module_path),
        Some(String::from("xrf-app")),
        "{module_path}"
      );
    }

    assert_eq!(
      modules.module_of("xrf_application::state"),
      Some(String::from("xrf-application"))
    );
  }
}
