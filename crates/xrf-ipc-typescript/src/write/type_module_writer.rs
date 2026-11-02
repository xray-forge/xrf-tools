//! Writing `core/ipc/types/`, one module per crate that declares an exported type or constant.

use std::collections::{BTreeMap, BTreeSet};
use std::iter;
use std::path::Path;

use specta::datatype::{DataType, NamedDataType};
use specta::{Format, Types};
use specta_typescript::{Exporter, primitives};
use xrf_ipc::IpcConstant;

use crate::collected::CollectedConstants;
use crate::collected::CommandTypescript;
use crate::enumerations::Enumerations;
use crate::input::MirrorImports;
use crate::input::TypeModules;
use crate::typescript::format::TypeScriptFormat;
use crate::typescript::syntax::render_value;
use crate::write::GeneratedOutput;
use crate::write::TypeOwnership;

/// Writes `types/`, one module per crate that declares an exported type or constant.
pub(crate) struct TypeModuleWriter;

impl TypeModuleWriter {
  /// The largest integer a JavaScript number holds exactly, `Number.MAX_SAFE_INTEGER`.
  const MAX_SAFE_INTEGER: u64 = (1 << 53) - 1;

  /// Writes one module per declaring crate, or crate family as `type_modules` groups them, each importing another as
  /// `imports` addresses it, answering with the ownership and the enums those modules establish.
  pub(crate) fn write_all(
    output: &Path,
    collected: &Types,
    constants: &CollectedConstants,
    type_modules: &TypeModules,
    imports: &MirrorImports,
  ) -> (TypeOwnership, Enumerations) {
    let types: Types = TypeScriptFormat::default()
      .map_types(collected)
      .expect("Failed to apply the TypeScript format to the collected types")
      .into_owned();
    let exporter: Exporter = CommandTypescript::configured().into();
    let enumerations: Enumerations = Enumerations::resolve(collected, &types);
    let (mut ownership, modules) = TypeOwnership::resolve(&types, type_modules, imports);

    for (module, named) in &modules {
      for declared in named {
        if let Some(name) = enumerations.enum_name_of(&declared.name) {
          ownership.declare(name, module);
        }
      }
    }

    let mut graph: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut rendered: BTreeMap<String, String> = BTreeMap::new();

    for module in modules.keys().chain(constants.modules()).collect::<BTreeSet<&String>>() {
      let declarations: String = modules
        .get(module)
        .into_iter()
        .flatten()
        .map(|declared| Self::render_declaration(&exporter, &types, &enumerations, declared, module))
        .collect::<Vec<String>>()
        .join("\n");
      let exported: String = constants
        .of_module(module)
        .map(|(constant, data_type)| Self::render_constant(constant, data_type, module, &ownership, &exporter, &types))
        .collect::<Vec<String>>()
        .join("\n");
      let source: String = format!("{exported}\n{declarations}");

      ownership.assert_no_foreign_references(&source, module);

      let referenced: BTreeSet<&str> = ownership.references(&source, module);

      graph.insert(module.clone(), ownership.modules_of(&referenced));

      // `primitives::export` renders declarations alone, so the file header the high-level exporter would have
      // written has to be prepended here.
      let imports: String = ownership.imports(&referenced);
      let separator: &str = if imports.is_empty() { "" } else { "\n" };
      let constant_separator: &str = if exported.is_empty() || declarations.is_empty() {
        ""
      } else {
        "\n"
      };

      rendered.insert(
        module.clone(),
        format!(
          "{}\n{imports}{separator}{exported}{constant_separator}{declarations}",
          GeneratedOutput::HEADER
        ),
      );
    }

    TypeOwnership::assert_no_import_cycles(&graph);

    for (module, contents) in rendered {
      GeneratedOutput::write(&output.join(format!("{module}.ts")), &contents);
    }

    (ownership, enumerations)
  }

  /// One type as the frontend declares it, which is Specta's rendering, its doc indentation normalized, plus whatever
  /// enum the type declares.
  fn render_declaration(
    exporter: &Exporter,
    types: &Types,
    enumerations: &Enumerations,
    declared: &NamedDataType,
    module: &str,
  ) -> String {
    let union: String = primitives::export(&exporter, types, iter::once(declared), "")
      .unwrap_or_else(|error| panic!("Failed to export {module} type {}: {error}", declared.name));

    GeneratedOutput::normalize_doc_indentation(&enumerations.render(declared, &union))
  }

  /// One constant as the frontend declares it: its doc comment, then `export const NAME: type = value;`.
  fn render_constant(
    constant: &IpcConstant,
    data_type: &DataType,
    module: &str,
    ownership: &TypeOwnership,
    exporter: &Exporter,
    types: &Types,
  ) -> String {
    // Another module's type of the name would be imported into this one, where it collides with the constant.
    if let Some(owner) = ownership.owner_of(constant.name) {
      panic!(
        "Constant `{}` of {module} takes the name of a type {owner} declares",
        constant.name
      );
    }

    let value: serde_json::Value = (constant.value)().unwrap_or_else(|error| {
      panic!(
        "Constant `{}` of `{}` does not serialize: {error}",
        constant.name, constant.module_path
      )
    });
    Self::assert_exact_numbers(constant, &value);

    let doc: String = Self::render_doc(constant.doc);
    let literal: String = render_value(&value);

    let kind: String = match &value {
      // Its own literal, by assertion rather than annotation, as the frontend's lint asks of a literal type.
      serde_json::Value::String(_) => return format!("{doc}export const {} = {literal} as const;\n", constant.name),
      serde_json::Value::Number(_) => String::from("number"),
      serde_json::Value::Bool(_) => String::from("boolean"),
      serde_json::Value::Object(_) | serde_json::Value::Array(_) => {
        Self::render_type(constant, data_type, exporter, types)
      }
      serde_json::Value::Null => panic!(
        "Constant `{}` of {module} serializes to null, which the bindings cannot type",
        constant.name
      ),
    };

    format!("{doc}export const {}: {kind} = {literal};\n", constant.name)
  }

  /// Fails on an integer, at any depth of `value`, that a JavaScript number does not hold exactly.
  fn assert_exact_numbers(constant: &IpcConstant, value: &serde_json::Value) {
    match value {
      serde_json::Value::Object(fields) => {
        for field in fields.values() {
          Self::assert_exact_numbers(constant, field);
        }
      }
      serde_json::Value::Array(elements) => {
        for element in elements {
          Self::assert_exact_numbers(constant, element);
        }
      }
      serde_json::Value::Number(number) => assert!(
        number
          .as_i64()
          .map(i64::unsigned_abs)
          .or(number.as_u64())
          .is_none_or(|magnitude| magnitude <= Self::MAX_SAFE_INTEGER),
        "Constant `{}` of `{}` holds {number}, an integer a JavaScript number does not hold exactly",
        constant.name,
        constant.module_path
      ),
      _ => {}
    }
  }

  /// The TypeScript type of an object or array constant: a reference to a named type, or the inline shape of any other.
  fn render_type(constant: &IpcConstant, data_type: &DataType, exporter: &Exporter, types: &Types) -> String {
    match data_type {
      DataType::Reference(_) => primitives::reference(&exporter, types, data_type),
      other => primitives::inline(&exporter, types, other),
    }
    .unwrap_or_else(|error| {
      panic!(
        "Constant `{}` of `{}` has a type the bindings cannot name: {error}",
        constant.name, constant.module_path
      )
    })
  }

  /// A doc comment as JSDoc, on one line where it is one line; nothing where there is none.
  fn render_doc(doc: &str) -> String {
    let escaped: String = doc.replace("*/", "*\\/");
    let lines: Vec<&str> = escaped.lines().collect();

    match lines.as_slice() {
      [] => String::new(),
      [line] => format!("/** {line} */\n"),
      lines => {
        let body: String = lines
          .iter()
          .map(|line| {
            if line.is_empty() {
              String::from(" *\n")
            } else {
              format!(" * {line}\n")
            }
          })
          .collect();

        format!("/**\n{body} */\n")
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use std::fs;
  use std::path::PathBuf;

  use serde::Serialize;
  use specta::datatype::DataType;
  use specta::{Format, Types};
  use specta_typescript::Exporter;
  use xrf_ipc::IpcConstant;
  use xrf_test_utils::GeneratedResources;

  use crate::collected::CollectedConstants;
  use crate::collected::CommandTypescript;
  use crate::input::MirrorImports;
  use crate::input::TypeModules;
  use crate::typescript::format::TypeScriptFormat;
  use crate::write::TypeModuleWriter;
  use crate::write::TypeOwnership;

  const IMPORTS: MirrorImports = MirrorImports::new("@/fixture");

  /// One constant rendered alone, into a module declaring nothing but the types its own type references.
  fn render(constant: &IpcConstant) -> String {
    let mut collected: Types = Types::default();
    let data_type: DataType = (constant.data_type)(&mut collected);
    let types: Types = TypeScriptFormat::default()
      .map_types(&collected)
      .expect("types to map")
      .into_owned();
    let (ownership, _) = TypeOwnership::resolve(&types, &TypeModules::default(), &IMPORTS);
    let exporter: Exporter = CommandTypescript::configured().into();

    TypeModuleWriter::render_constant(constant, &data_type, "xrf-fixture", &ownership, &exporter, &types)
  }

  /// Limits a test exports as one value.
  #[derive(Serialize, specta::Type)]
  #[serde(rename_all = "camelCase")]
  struct ExportedLimits {
    /// Spellings the test admits.
    names: &'static [&'static str],
    most_steps: u32,
  }

  /// The limits the test exports, whose type no command carries.
  #[xrf_ipc::constant]
  const EXPORTED_LIMITS: ExportedLimits = ExportedLimits {
    names: &["steps", "two words"],
    most_steps: 7,
  };

  /// Steps a test exports to its crate's module.
  ///
  /// The second line closes no comment: */
  #[xrf_ipc::constant]
  const EXPORTED_STEPS: u32 = 7;

  /// A name the test exports beside the steps.
  #[xrf_ipc::constant]
  const EXPORTED_NAME: &str = "steps";

  /// A pattern the test exports as written:
  ///   its indent and its ` *  ` kept.
  #[xrf_ipc::constant]
  const EXPORTED_PATTERN: &str = "a *  b";

  #[test]
  fn writes_a_marked_constant_into_its_crate_s_module_with_its_doc() {
    let output: PathBuf = GeneratedResources::path("type-module-writer");

    fs::create_dir_all(&output).expect("directory to be created");

    let mut types: Types = Types::default();
    let constants: CollectedConstants = CollectedConstants::collect(&TypeModules::default(), &mut types);

    TypeModuleWriter::write_all(&output, &types, &constants, &TypeModules::default(), &IMPORTS);

    let module: String = fs::read_to_string(output.join("xrf-ipc-typescript.ts")).expect("module to be written");

    assert_eq!(EXPORTED_STEPS, 7);
    assert_eq!(EXPORTED_NAME, "steps");
    assert_eq!(EXPORTED_PATTERN, "a *  b");
    assert_eq!(
      (EXPORTED_LIMITS.names, EXPORTED_LIMITS.most_steps),
      (&["steps", "two words"][..], 7)
    );
    assert_eq!(
      module,
      "// Auto-generated rust bindings. Do not edit it manually.\n\n\
       /** The limits the test exports, whose type no command carries. */\n\
       export const EXPORTED_LIMITS: ExportedLimits = { mostSteps: 7, names: [\"steps\", \"two words\"] };\n\n\
       /** A name the test exports beside the steps. */\n\
       export const EXPORTED_NAME = \"steps\" as const;\n\n\
       /**\n \
       * A pattern the test exports as written:\n \
       *   its indent and its ` *  ` kept.\n \
       */\n\
       export const EXPORTED_PATTERN = \"a *  b\" as const;\n\n\
       /**\n \
       * Steps a test exports to its crate's module.\n \
       *\n \
       * The second line closes no comment: *\\/\n \
       */\n\
       export const EXPORTED_STEPS: number = 7;\n\n\
       /** Limits a test exports as one value. */\n\
       export type ExportedLimits = {\n\
       \t/** Spellings the test admits. */\n\
       \tnames: string[],\n\
       \tmostSteps: number,\n\
       };\n"
    );
  }

  #[test]
  fn writes_an_f32_in_its_shortest_form() {
    let rendered: String = render(&IpcConstant {
      module_path: "xrf_fixture::limits",
      name: "SHARE",
      doc: "",
      value: || IpcConstant::serialize_value(&0.1_f32),
      data_type: |types| IpcConstant::define_type(&0.1_f32, types),
    });

    assert_eq!(rendered, "export const SHARE: number = 0.1;\n");
  }

  #[test]
  fn writes_the_largest_integer_a_javascript_number_holds() {
    let rendered: String = render(&IpcConstant {
      module_path: "xrf_fixture::limits",
      name: "LARGEST",
      doc: "",
      value: || IpcConstant::serialize_value(&-9_007_199_254_740_991_i64),
      data_type: |types| IpcConstant::define_type(&0_i64, types),
    });

    assert_eq!(rendered, "export const LARGEST: number = -9007199254740991;\n");
  }

  #[test]
  #[should_panic(expected = "Constant `HUGE` of `xrf_fixture::limits` holds 9007199254740992, an integer a JavaScript")]
  fn refuses_an_integer_a_javascript_number_does_not_hold() {
    render(&IpcConstant {
      module_path: "xrf_fixture::limits",
      name: "HUGE",
      doc: "",
      value: || IpcConstant::serialize_value(&9_007_199_254_740_992_u64),
      data_type: |types| IpcConstant::define_type(&0_u64, types),
    });
  }

  #[test]
  #[should_panic(
    expected = "Constant `NESTED` of `xrf_fixture::limits` holds 9007199254740992, an integer a JavaScript"
  )]
  fn refuses_an_integer_a_javascript_number_does_not_hold_inside_an_array() {
    render(&IpcConstant {
      module_path: "xrf_fixture::limits",
      name: "NESTED",
      doc: "",
      value: || IpcConstant::serialize_value(&[1_u64, 9_007_199_254_740_992_u64]),
      data_type: |types| IpcConstant::define_type(&[0_u64; 2], types),
    });
  }

  #[test]
  fn writes_an_array_typed_as_its_elements() {
    let rendered: String = render(&IpcConstant {
      module_path: "xrf_fixture::limits",
      name: "STEPS",
      doc: "",
      value: || IpcConstant::serialize_value(&[1_u32, 2, 3]),
      data_type: |types| IpcConstant::define_type::<[u32]>(&[1, 2, 3], types),
    });

    assert_eq!(rendered, "export const STEPS: number[] = [1, 2, 3];\n");
  }

  #[test]
  #[should_panic(
    expected = "Constant `ExportedLimits` of xrf-fixture takes the name of a type xrf-ipc-typescript declares"
  )]
  fn refuses_a_constant_named_as_a_type_of_another_module() {
    render(&IpcConstant {
      module_path: "xrf_fixture::limits",
      name: "ExportedLimits",
      doc: "",
      value: || IpcConstant::serialize_value(&EXPORTED_LIMITS),
      data_type: |types| IpcConstant::define_type(&EXPORTED_LIMITS, types),
    });
  }

  #[test]
  #[should_panic(expected = "Constant `BROKEN` of `xrf_fixture::limits` does not serialize")]
  fn names_a_constant_that_does_not_serialize() {
    render(&IpcConstant {
      module_path: "xrf_fixture::limits",
      name: "BROKEN",
      doc: "",
      value: || serde_json::from_str("not a value"),
      data_type: |types| IpcConstant::define_type(&0_u32, types),
    });
  }
}
