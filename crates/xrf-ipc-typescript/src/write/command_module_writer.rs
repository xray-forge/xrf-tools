//! Writing `commands/`, one module per Tauri plugin, its raw and bulk companions, and the signatures they all merge
//! into.

use std::collections::BTreeSet;
use std::path::Path;

use xrf_utils::format_path;

use crate::collected::{PluginCommands, SpectaCommand};
use crate::enumerations::Enumerations;
use crate::input::MirrorImports;
use crate::write::plugin_symbols::PluginSymbols;
use crate::write::wrapper_module::WrapperModule;
use crate::write::{GeneratedOutput, PluginModules, TypeOwnership};

/// Writes `commands/`, one module per Tauri plugin.
pub(crate) struct CommandModuleWriter;

impl CommandModuleWriter {
  /// Where Tauri's `Channel` is imported from, a transport handle the counted wrapper has no business in.
  const CHANNEL_IMPORT: &'static str = "import { Channel } from \"@tauri-apps/api/core\";";

  /// Writes one plugin's command module: its commands' enum, the wrappers the export rendered invoking each by its
  /// member, and the commands' signatures, importing the types the wrappers name from the modules declaring them.
  ///
  /// The command object is named after the plugin, so no call site has to alias one of a dozen identical names.
  /// `invoke` is the counted wrapper rather than Tauri's own: it counts every call, which is why it is imported here
  /// rather than left to every call site to remember, so a command added later is measured by existing.
  ///
  /// # Panics
  ///
  /// Panics on a plugin with no typed commands, and when the module cannot be written.
  pub(crate) fn write(
    path: &Path,
    plugin: &str,
    exported: &PluginCommands,
    ownership: &TypeOwnership,
    enumerations: &Enumerations,
    imports: &MirrorImports,
  ) {
    let commands: &[SpectaCommand] = &exported.commands;

    assert!(
      !commands.is_empty(),
      "Plugin `{plugin}` exports no typed commands, so {} would declare none",
      format_path(path)
    );

    let symbols: PluginSymbols = PluginSymbols::new(plugin);
    let wrappers: String = enumerations.rewrite_parameters(&exported.wrappers);

    ownership.assert_no_foreign_references(&wrappers, plugin);

    let type_imports: String = ownership.imports_for(&wrappers);
    let channel_import: String = if exported.uses_channel {
      format!("{}\n", Self::CHANNEL_IMPORT)
    } else {
      String::new()
    };
    let enumeration: String = Self::render_enum(
      &format!("The wire name of every command of the `{plugin}` plugin."),
      &symbols.command_enum(),
      commands
        .iter()
        .map(|command| (command.name.as_str(), format!("plugin:{plugin}|{}", command.name))),
    );
    let signatures: String = Self::render_signatures(
      &format!("What each command of the `{plugin}` plugin takes and resolves to, keyed by its wire name."),
      &symbols.command_signatures(),
      &symbols.command_enum(),
      &symbols.commands(),
      "IIpcCommandSignature",
      commands.iter().map(|command| {
        (
          command.name.as_str(),
          command.arguments.iter().map(String::as_str).collect(),
        )
      }),
    );

    GeneratedOutput::write(
      path,
      &format!(
        "{}\nimport {{ IIpcCommandSignature }} from \"{}\";\n{type_imports}{channel_import}\
         import {{ invoke as __TAURI_INVOKE }} from \"{}\";\n\n{enumeration}\n/** Commands */\n\
         export const {} = {wrappers};\n\n{signatures}",
        GeneratedOutput::HEADER,
        imports.signature(),
        imports.invoke(),
        symbols.commands(),
      ),
    );
  }

  /// Writes the wrappers for one domain's raw commands.
  ///
  /// Specta cannot collect a command returning `tauri::ipc::Response`, so these are generated from the registry
  /// instead of written by hand. The registry carries each argument's TypeScript type for exactly this reason;
  /// the return is always `ArrayBuffer`, which is what the raw response arrives as.
  pub(crate) fn write_raw(
    path: &Path,
    plugin: &str,
    commands: &[(&str, &[(&str, &str)])],
    ownership: &TypeOwnership,
    enumerations: &Enumerations,
    imports: &MirrorImports,
  ) {
    if commands.is_empty() {
      return;
    }

    Self::assert_distinct(path, commands.iter().map(|(wire_name, _)| *wire_name));

    let symbols: PluginSymbols = PluginSymbols::new(plugin);
    let wrappers: String = commands
      .iter()
      .map(|(wire_name, arguments)| {
        let (parameters, payload): (String, String) = Self::render_arguments(arguments);

        format!(
          "  {}: ({parameters}): Promise<ArrayBuffer> =>\n    invokeRaw({}.{}, {{ {payload} }}),\n",
          PluginSymbols::wrapper(wire_name),
          symbols.raw_command_enum(),
          PluginSymbols::member(wire_name)
        )
      })
      .collect();

    WrapperModule {
      import: format!(
        "import {{ invokeRaw }} from \"{}\";\nimport {{ IIpcCommandSignature }} from \"{}\";",
        imports.raw(),
        imports.signature()
      ),
      enumeration: Self::render_enum(
        &format!("The wire name of every raw command of the `{plugin}` plugin."),
        &symbols.raw_command_enum(),
        commands
          .iter()
          .map(|(wire_name, _)| (*wire_name, format!("plugin:{plugin}|{wire_name}"))),
      ),
      declaration: format!(
        "/** Commands answering with raw bytes over IPC, which Specta cannot type. */\nexport const {}",
        symbols.raw_commands()
      ),
      wrappers,
      signatures: Self::render_signatures(
        &format!("What each raw command of the `{plugin}` plugin takes and resolves to, keyed by its wire name."),
        &symbols.raw_command_signatures(),
        &symbols.raw_command_enum(),
        &symbols.raw_commands(),
        "IIpcCommandSignature",
        Self::declared_arguments(commands),
      ),
    }
    .write(path, plugin, ownership, enumerations);
  }

  /// Writes the wrappers for one domain's bulk routes.
  ///
  /// A route is not a command and Specta never sees it, so these are generated from the registry instead of written
  /// by hand; the registry carries each argument's TypeScript type for exactly this reason. A wrapper answers the call
  /// as data, its route and arguments, so the page can fetch it or hand it to another thread to fetch.
  pub(crate) fn write_bulk(
    path: &Path,
    plugin: &str,
    routes: &[(&str, &[(&str, &str)])],
    ownership: &TypeOwnership,
    enumerations: &Enumerations,
    imports: &MirrorImports,
  ) {
    if routes.is_empty() {
      return;
    }

    Self::assert_distinct(path, routes.iter().map(|(route, _)| *route));

    let symbols: PluginSymbols = PluginSymbols::new(plugin);
    let wrappers: String = routes
      .iter()
      .map(|(route, arguments)| {
        let (parameters, payload): (String, String) = Self::render_arguments(arguments);

        format!(
          "  {}: ({parameters}): IBulkCall => ({{ args: {{ {payload} }}, route: {}.{} }}),\n",
          PluginSymbols::wrapper(route),
          symbols.bulk_route_enum(),
          PluginSymbols::member(route)
        )
      })
      .collect();

    WrapperModule {
      import: format!(
        "import {{ IBulkCall }} from \"{}\";\nimport {{ IIpcBulkRouteSignature }} from \"{}\";",
        imports.bulk(),
        imports.signature()
      ),
      enumeration: Self::render_enum(
        &format!("Every bulk route of the `{plugin}` plugin, as the transport serves it."),
        &symbols.bulk_route_enum(),
        routes.iter().map(|(route, _)| (*route, format!("{plugin}/{route}"))),
      ),
      declaration: format!(
        "/** Routes answering with bytes over the loopback transport, which Specta never sees. */\nexport const {}",
        symbols.bulk_routes()
      ),
      wrappers,
      signatures: Self::render_signatures(
        &format!("What each bulk route of the `{plugin}` plugin takes, keyed by its route."),
        &symbols.bulk_route_signatures(),
        &symbols.bulk_route_enum(),
        &symbols.bulk_routes(),
        "IIpcBulkRouteSignature",
        Self::declared_arguments(routes),
      ),
    }
    .write(path, plugin, ownership, enumerations);
  }

  /// Writes `signatures.ts`: every plugin's command signatures merged into one interface, and every plugin's bulk
  /// route signatures into another, so a test or a tool names any command by its enum and gets its types.
  pub(crate) fn write_signatures(path: &Path, plugins: &[PluginModules], imports: &MirrorImports) {
    let mut import_lines: Vec<String> = Vec::new();
    let mut commands: Vec<String> = Vec::new();
    let mut routes: Vec<String> = Vec::new();

    for PluginModules {
      name: plugin,
      has_raw,
      has_bulk,
    } in plugins
    {
      let symbols: PluginSymbols = PluginSymbols::new(plugin);

      import_lines.push(format!(
        "import {{ {} }} from \"{}\";",
        symbols.command_signatures(),
        imports.command_module(plugin)
      ));
      commands.push(symbols.command_signatures());

      if *has_raw {
        import_lines.push(format!(
          "import {{ {} }} from \"{}\";",
          symbols.raw_command_signatures(),
          imports.command_module(&format!("{plugin}-raw"))
        ));
        commands.push(symbols.raw_command_signatures());
      }

      if *has_bulk {
        import_lines.push(format!(
          "import {{ {} }} from \"{}\";",
          symbols.bulk_route_signatures(),
          imports.command_module(&format!("{plugin}-bulk"))
        ));
        routes.push(symbols.bulk_route_signatures());
      }
    }

    import_lines.sort();

    GeneratedOutput::write(
      path,
      &format!(
        "{}\n{}\n\n\
         /** Every command any plugin answers, raw ones included, keyed by its wire name. */\n\
         export interface IIpcCommandSignatures{} {{}}\n\n\
         /** Every bulk route any plugin serves, keyed by its route. */\n\
         export interface IIpcBulkRouteSignatures{} {{}}\n",
        GeneratedOutput::HEADER,
        import_lines.join("\n"),
        Self::render_extends(&commands),
        Self::render_extends(&routes)
      ),
    );
  }

  /// A registry declaration's arguments as a TypeScript parameter list and the object shorthand passing them on.
  fn render_arguments(arguments: &[(&str, &str)]) -> (String, String) {
    let parameters: String = arguments
      .iter()
      .map(|(name, argument_type)| format!("{name}: {argument_type}"))
      .collect::<Vec<String>>()
      .join(", ");
    let payload: String = arguments
      .iter()
      .map(|(name, _)| (*name).to_string())
      .collect::<Vec<String>>()
      .join(", ");

    (parameters, payload)
  }

  /// A registry's declarations as each one's name and its arguments' names, which is what a signature reads.
  fn declared_arguments<'a>(
    declarations: &'a [(&'a str, &'a [(&'a str, &'a str)])],
  ) -> impl Iterator<Item = (&'a str, Vec<&'a str>)> {
    declarations
      .iter()
      .map(|(name, arguments)| (*name, arguments.iter().map(|(argument, _)| *argument).collect()))
  }

  /// An exported string enum under `doc`, one member per `(name, value)`, its member named after the name.
  fn render_enum<'a>(doc: &str, name: &str, members: impl Iterator<Item = (&'a str, String)>) -> String {
    let members: String = members
      .map(|(member, value)| format!("  {} = \"{value}\",\n", PluginSymbols::member(member)))
      .collect();

    format!("/** {doc} */\nexport enum {name} {{\n{members}}}\n")
  }

  /// An exported interface under `doc` keying each declaration by its enum member, its signature read off the
  /// wrapper of the same name in `object` through `signature`, with its arguments named in order.
  fn render_signatures<'a>(
    doc: &str,
    name: &str,
    enumeration: &str,
    object: &str,
    signature: &str,
    declarations: impl Iterator<Item = (&'a str, Vec<&'a str>)>,
  ) -> String {
    let fields: String = declarations
      .map(|(declaration, arguments)| {
        let names: String = arguments
          .iter()
          .map(|argument| format!("\"{argument}\""))
          .collect::<Vec<String>>()
          .join(", ");

        format!(
          "  [{enumeration}.{}]: {signature}<typeof {object}.{}, [{names}]>;\n",
          PluginSymbols::member(declaration),
          PluginSymbols::wrapper(declaration)
        )
      })
      .collect();

    format!("/** {doc} */\nexport interface {name} {{\n{fields}}}\n")
  }

  /// ` extends A, B` for the interfaces `names`, nothing for none.
  fn render_extends(names: &[String]) -> String {
    if names.is_empty() {
      String::new()
    } else {
      format!(" extends {}", names.join(", "))
    }
  }

  /// Fails when two declarations of one module fold to one enum member, which TypeScript would reject as a duplicate
  /// identifier only once the frontend compiles.
  fn assert_distinct<'a>(path: &Path, names: impl Iterator<Item = &'a str>) {
    let mut members: BTreeSet<String> = BTreeSet::new();

    for name in names {
      assert!(
        members.insert(PluginSymbols::member(name)),
        "{} declares `{name}` twice",
        format_path(path)
      );
    }
  }
}

#[cfg(test)]
mod tests {
  use std::fs;
  use std::path::PathBuf;

  use specta::Types;
  use xrf_test_utils::GeneratedResources;

  use crate::collected::{PluginCommands, SpectaCommand};
  use crate::enumerations::Enumerations;
  use crate::input::{MirrorImports, TypeModules};
  use crate::write::{CommandModuleWriter, PluginModules, TypeOwnership};

  const IMPORTS: MirrorImports = MirrorImports::new("#");

  /// Writes one kind of wrapper module with nothing collected, answering where it went.
  fn written(name: &str, write: impl FnOnce(&PathBuf, &TypeOwnership, &Enumerations)) -> PathBuf {
    let types: Types = Types::default();
    let (ownership, _): (TypeOwnership, _) = TypeOwnership::resolve(&types, &TypeModules::default(), &IMPORTS);
    let enumerations: Enumerations = Enumerations::resolve(&types, &types);
    let path: PathBuf = GeneratedResources::path("command-module-writer").join(name);

    fs::create_dir_all(path.parent().expect("module to have a directory")).expect("directory to be created");

    write(&path, &ownership, &enumerations);

    path
  }

  /// The commands a fixture plugin exports, its wrappers as the export renders them.
  fn exported(uses_channel: bool) -> PluginCommands {
    PluginCommands {
      commands: vec![
        SpectaCommand {
          name: String::from("open_thing"),
          arguments: vec![String::from("sessionId"), String::from("jobId")],
        },
        SpectaCommand {
          name: String::from("list_things"),
          arguments: Vec::new(),
        },
      ],
      wrappers: String::from(
        "{\n\topenThing: (sessionId: string, jobId: string) => __TAURI_INVOKE<number>(EFixtureCommand.OPEN_THING, { \
         sessionId, jobId }),\n\tlistThings: () => __TAURI_INVOKE<string[]>(EFixtureCommand.LIST_THINGS),\n}",
      ),
      uses_channel,
    }
  }

  #[test]
  fn a_typed_module_names_its_commands_by_enum_and_declares_their_signatures() {
    let path: PathBuf = written("fixture.ts", |path, ownership, enumerations| {
      CommandModuleWriter::write(path, "fixture", &exported(false), ownership, enumerations, &IMPORTS);
    });

    assert_eq!(
      fs::read_to_string(&path).expect("module to be written"),
      "// Auto-generated rust bindings. Do not edit it manually.\n\n\
       import { IIpcCommandSignature } from \"#/signature\";\n\
       import { invoke as __TAURI_INVOKE } from \"#/invoke\";\n\n\
       /** The wire name of every command of the `fixture` plugin. */\n\
       export enum EFixtureCommand {\n  \
       OPEN_THING = \"plugin:fixture|open_thing\",\n  \
       LIST_THINGS = \"plugin:fixture|list_things\",\n\
       }\n\n\
       /** Commands */\n\
       export const fixtureCommands = {\n\t\
       openThing: (sessionId: string, jobId: string) => __TAURI_INVOKE<number>(EFixtureCommand.OPEN_THING, { sessionId, jobId }),\n\t\
       listThings: () => __TAURI_INVOKE<string[]>(EFixtureCommand.LIST_THINGS),\n\
       };\n\n\
       /** What each command of the `fixture` plugin takes and resolves to, keyed by its wire name. */\n\
       export interface IFixtureCommandSignatures {\n  \
       [EFixtureCommand.OPEN_THING]: IIpcCommandSignature<typeof fixtureCommands.openThing, [\"sessionId\", \"jobId\"]>;\n  \
       [EFixtureCommand.LIST_THINGS]: IIpcCommandSignature<typeof fixtureCommands.listThings, []>;\n\
       }\n"
    );
  }

  #[test]
  fn a_module_whose_wrappers_carry_a_channel_imports_it_from_tauri() {
    let path: PathBuf = written("channel.ts", |path, ownership, enumerations| {
      CommandModuleWriter::write(path, "fixture", &exported(true), ownership, enumerations, &IMPORTS);
    });

    assert!(fs::read_to_string(&path).expect("module to be written").contains(
      "import { Channel } from \"@tauri-apps/api/core\";\nimport { invoke as __TAURI_INVOKE } from \"#/invoke\";\n"
    ));
  }

  #[test]
  fn a_raw_command_is_written_as_a_wrapper_invoking_it_raw() {
    let path: PathBuf = written("fixture-raw.ts", |path, ownership, enumerations| {
      CommandModuleWriter::write_raw(
        path,
        "fixture",
        &[("read_bytes", &[("sessionId", "string"), ("length", "number")])],
        ownership,
        enumerations,
        &IMPORTS,
      );
    });

    assert_eq!(
      fs::read_to_string(&path).expect("module to be written"),
      "// Auto-generated rust bindings. Do not edit it manually.\n\n\
       import { invokeRaw } from \"#/raw\";\n\
       import { IIpcCommandSignature } from \"#/signature\";\n\n\
       /** The wire name of every raw command of the `fixture` plugin. */\n\
       export enum EFixtureRawCommand {\n  \
       READ_BYTES = \"plugin:fixture|read_bytes\",\n\
       }\n\n\
       /** Commands answering with raw bytes over IPC, which Specta cannot type. */\n\
       export const fixtureRawCommands = {\n  \
       readBytes: (sessionId: string, length: number): Promise<ArrayBuffer> =>\n    \
       invokeRaw(EFixtureRawCommand.READ_BYTES, { sessionId, length }),\n\
       };\n\n\
       /** What each raw command of the `fixture` plugin takes and resolves to, keyed by its wire name. */\n\
       export interface IFixtureRawCommandSignatures {\n  \
       [EFixtureRawCommand.READ_BYTES]: IIpcCommandSignature<typeof fixtureRawCommands.readBytes, [\"sessionId\", \"length\"]>;\n\
       }\n"
    );
  }

  #[test]
  fn a_bulk_route_is_written_as_a_wrapper_answering_the_call() {
    let path: PathBuf = written("fixture-bulk.ts", |path, ownership, enumerations| {
      CommandModuleWriter::write_bulk(
        path,
        "fixture",
        &[("read_bytes", &[("length", "number")])],
        ownership,
        enumerations,
        &IMPORTS,
      );
    });

    assert_eq!(
      fs::read_to_string(&path).expect("module to be written"),
      "// Auto-generated rust bindings. Do not edit it manually.\n\n\
       import { IBulkCall } from \"#/bulk/bulk-call\";\n\
       import { IIpcBulkRouteSignature } from \"#/signature\";\n\n\
       /** Every bulk route of the `fixture` plugin, as the transport serves it. */\n\
       export enum EFixtureBulkRoute {\n  \
       READ_BYTES = \"fixture/read_bytes\",\n\
       }\n\n\
       /** Routes answering with bytes over the loopback transport, which Specta never sees. */\n\
       export const fixtureBulkRoutes = {\n  \
       readBytes: (length: number): IBulkCall => ({ args: { length }, route: EFixtureBulkRoute.READ_BYTES }),\n\
       };\n\n\
       /** What each bulk route of the `fixture` plugin takes, keyed by its route. */\n\
       export interface IFixtureBulkRouteSignatures {\n  \
       [EFixtureBulkRoute.READ_BYTES]: IIpcBulkRouteSignature<typeof fixtureBulkRoutes.readBytes, [\"length\"]>;\n\
       }\n"
    );
  }

  #[test]
  #[should_panic(expected = "declares `read_bytes` twice")]
  fn a_route_declared_twice_fails_rather_than_writing_one_member_twice() {
    written("twice-bulk.ts", |path, ownership, enumerations| {
      CommandModuleWriter::write_bulk(
        path,
        "fixture",
        &[("read_bytes", &[]), ("read_bytes", &[("length", "number")])],
        ownership,
        enumerations,
        &IMPORTS,
      );
    });
  }

  #[test]
  #[should_panic(expected = "Plugin `fixture` exports no typed commands")]
  fn a_plugin_without_typed_commands_fails_rather_than_writing_an_empty_module() {
    written("empty.ts", |path, ownership, enumerations| {
      let empty: PluginCommands = PluginCommands {
        commands: Vec::new(),
        wrappers: String::from("{}"),
        uses_channel: false,
      };

      CommandModuleWriter::write(path, "fixture", &empty, ownership, enumerations, &IMPORTS);
    });
  }

  #[test]
  fn a_plugin_without_raw_commands_gets_no_raw_module() {
    let path: PathBuf = written("empty-raw.ts", |path, ownership, enumerations| {
      CommandModuleWriter::write_raw(path, "empty", &[], ownership, enumerations, &IMPORTS);
    });

    assert!(!path.exists());
  }

  #[test]
  fn the_signatures_merge_every_plugin_s_modules() {
    let path: PathBuf = written("signatures.ts", |path, _, _| {
      CommandModuleWriter::write_signatures(
        path,
        &[
          PluginModules {
            name: "jobs",
            has_raw: false,
            has_bulk: false,
          },
          PluginModules {
            name: "archives",
            has_raw: false,
            has_bulk: true,
          },
        ],
        &IMPORTS,
      );
    });

    assert_eq!(
      fs::read_to_string(&path).expect("module to be written"),
      "// Auto-generated rust bindings. Do not edit it manually.\n\n\
       import { IArchivesBulkRouteSignatures } from \"#/commands/archives-bulk\";\n\
       import { IArchivesCommandSignatures } from \"#/commands/archives\";\n\
       import { IJobsCommandSignatures } from \"#/commands/jobs\";\n\n\
       /** Every command any plugin answers, raw ones included, keyed by its wire name. */\n\
       export interface IIpcCommandSignatures extends IJobsCommandSignatures, IArchivesCommandSignatures {}\n\n\
       /** Every bulk route any plugin serves, keyed by its route. */\n\
       export interface IIpcBulkRouteSignatures extends IArchivesBulkRouteSignatures {}\n"
    );
  }
}
