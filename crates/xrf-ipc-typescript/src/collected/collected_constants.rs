use std::collections::BTreeMap;

use specta::Types;
use specta::datatype::DataType;
use xrf_ipc::IpcConstant;

use crate::type_modules::TypeModules;

/// Every constant marked `#[xrf_ipc::constant]`, by the bindings module it lands in, each beside its type's definition.
pub(crate) struct CollectedConstants {
  /// Module to its constants, sorted by name.
  modules: BTreeMap<String, Vec<(&'static IpcConstant, DataType)>>,
}

impl CollectedConstants {
  /// Collects every constant linked into the binary, registering each one's type into `types` with the named types
  /// that one references, so a type only a constant carries is declared like one a command carries.
  ///
  /// # Panics
  ///
  /// Panics on a constant no workspace crate declares, and on two of one name in one module.
  pub(crate) fn collect(type_modules: &TypeModules, types: &mut Types) -> Self {
    Self::of(IpcConstant::list(), type_modules, types)
  }

  /// The same over `constants` rather than every one registered.
  fn of(
    constants: impl IntoIterator<Item = &'static IpcConstant>,
    type_modules: &TypeModules,
    types: &mut Types,
  ) -> Self {
    let mut modules: BTreeMap<String, Vec<(&IpcConstant, DataType)>> = BTreeMap::new();

    for constant in constants {
      let module: String = type_modules.module_of(constant.module_path).unwrap_or_else(|| {
        panic!(
          "Constant `{}` is declared in `{}`, outside the workspace, and has no module to live in",
          constant.name, constant.module_path
        )
      });

      modules
        .entry(module)
        .or_default()
        .push((constant, (constant.data_type)(types)));
    }

    for (module, constants) in &mut modules {
      constants.sort_by_key(|(constant, _)| constant.name);

      for pair in constants.windows(2) {
        let ((first, _), (second, _)) = (&pair[0], &pair[1]);

        assert_ne!(
          first.name, second.name,
          "Constant `{}` is exported twice into {module}, from `{}` and `{}`",
          first.name, first.module_path, second.module_path
        );
      }
    }

    Self { modules }
  }

  /// Every module holding a constant.
  pub(crate) fn modules(&self) -> impl Iterator<Item = &String> {
    self.modules.keys()
  }

  /// The constants `module` holds, sorted by name, each beside its type.
  pub(crate) fn of_module(&self, module: &str) -> impl Iterator<Item = (&'static IpcConstant, &DataType)> {
    self
      .modules
      .get(module)
      .into_iter()
      .flatten()
      .map(|(constant, data_type)| (*constant, data_type))
  }
}

#[cfg(test)]
mod tests {
  use specta::Types;
  use xrf_ipc::IpcConstant;

  use super::CollectedConstants;
  use crate::type_modules::TypeModules;

  static FIRST: IpcConstant = IpcConstant {
    module_path: "xrf_fixture::limits",
    name: "LIMIT",
    doc: "",
    value: || xrf_ipc::to_value(&1_u32),
    data_type: |types| xrf_ipc::data_type_of(&1_u32, types),
  };

  static SECOND: IpcConstant = IpcConstant {
    module_path: "xrf_fixture::other",
    name: "LIMIT",
    doc: "",
    value: || xrf_ipc::to_value(&2_u32),
    data_type: |types| xrf_ipc::data_type_of(&2_u32, types),
  };

  #[test]
  #[should_panic(expected = "Constant `LIMIT` is exported twice into xrf-fixture")]
  fn refuses_two_constants_of_one_name_in_one_module() {
    CollectedConstants::of([&FIRST, &SECOND], &TypeModules::default(), &mut Types::default());
  }
}
