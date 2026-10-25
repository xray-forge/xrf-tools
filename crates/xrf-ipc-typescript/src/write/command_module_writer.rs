//! Writing `commands/`, one module per Tauri plugin, its raw and bulk companions, and the signatures they all merge
//! into.

use std::fs;
use std::path::Path;

use xrf_utils::format_path;

use std::collections::BTreeSet;

use crate::collected::SpectaCommand;
use crate::enumerations::Enumerations;
use crate::input::MirrorImports;
use crate::write::plugin_symbols::PluginSymbols;
use crate::write::{GeneratedOutput, PluginModules, TypeOwnership};

/// Writes `commands/`, one module per Tauri plugin.
pub(crate) struct CommandModuleWriter;

impl CommandModuleWriter {
  /// Marker Tauri Specta writes before the types it inlines into a command module.
  ///
  /// Everything after it is a copy of a declaration owned by the crate the type comes from, so it is the cut point
  /// where those copies are dropped in favour of imports.
  const TYPES_MARKER: &'static str = "/* Types */";

  /// Marker Tauri Specta writes before the command object, where the enum naming its commands goes.
  const COMMANDS_MARKER: &'static str = "/** Commands */";

  /// Tauri Specta's own invoke import, for a module that also carries a job command's channel.
  const SPECTA_INVOKE_IMPORT_WITH_CHANNEL: &'static str =
    "import { invoke as __TAURI_INVOKE, Channel } from \"@tauri-apps/api/core\";";

  /// The same import for a module with no channel, which is the only other spelling Specta writes.
  const SPECTA_INVOKE_IMPORT: &'static str = "import { invoke as __TAURI_INVOKE } from \"@tauri-apps/api/core\";";

  /// Specta's name for the command object, which every module would otherwise export under the same name.
  const SPECTA_COMMANDS: &'static str = "export const commands = {";

  /// Replaces the types Tauri Specta inlined into a command module with imports, names its commands, has each wrapper
  /// invoke its command by the plugin's enum, and declares the commands' signatures.
  ///
  /// The command object is renamed after the plugin, so no call site has to alias one of a dozen identical names.
  /// `invoke` is taken from the counted wrapper instead of Tauri: it counts every call, which is why the redirect is
  /// written here rather than left to every call site to remember, so a command added later is measured by existing.
  /// `Channel` still comes from Tauri, being a transport handle the wrapper has no business in.
  pub(crate) fn finalize(
    path: &Path,
    plugin: &str,
    commands: &[SpectaCommand],
    ownership: &TypeOwnership,
    enumerations: &Enumerations,
    imports: &MirrorImports,
  ) {
    let symbols: PluginSymbols = PluginSymbols::new(plugin);
    let counted_invoke_import: String = format!("import {{ invoke as __TAURI_INVOKE }} from \"{}\";", imports.invoke());
    let contents: String =
      fs::read_to_string(path).unwrap_or_else(|error| panic!("Failed to read {}: {error}", format_path(path)));
    let specta: &str = contents
      .split_once(Self::TYPES_MARKER)
      .map_or(contents.as_str(), |(before, _)| before)
      .trim_end();

    assert!(
      specta.contains(Self::SPECTA_COMMANDS),
      "{} declares no `{}`; Tauri Specta's command object spelling has changed",
      format_path(path),
      Self::SPECTA_COMMANDS
    );

    let mut wrappers: String = specta
      .replace(
        Self::SPECTA_COMMANDS,
        &format!("export const {} = {{", symbols.commands()),
      )
      // The wider spelling first: replacing the bare one first would leave its `, Channel` behind.
      .replace(
        Self::SPECTA_INVOKE_IMPORT_WITH_CHANNEL,
        &format!("import {{ Channel }} from \"@tauri-apps/api/core\";\n{counted_invoke_import}"),
      )
      .replace(Self::SPECTA_INVOKE_IMPORT, &counted_invoke_import);

    for command in commands {
      let literal: String = format!("\"plugin:{plugin}|{}\"", command.name);

      assert_eq!(
        wrappers.matches(&literal).count(),
        1,
        "{} does not invoke {literal} exactly once; Tauri Specta's wrapper spelling has changed",
        format_path(path)
      );

      wrappers = wrappers.replace(
        &literal,
        &format!("{}.{}", symbols.command_enum(), PluginSymbols::member(&command.name)),
      );
    }

    let wrappers: String = enumerations.rewrite_parameters(&wrappers);

    // A module still importing `invoke` from Tauri would be dispatched and never counted, and nothing downstream would
    // say so. Specta changing how it writes that import is the way this stops being true, so it is asserted here.
    assert!(
      wrappers.contains(&counted_invoke_import),
      "{} does not route invoke through the counted wrapper; Tauri Specta's import spelling has changed",
      format_path(path)
    );
    assert!(
      wrappers.contains(Self::COMMANDS_MARKER),
      "{} has no `{}` marker; Tauri Specta's module layout has changed",
      format_path(path),
      Self::COMMANDS_MARKER
    );

    ownership.assert_no_foreign_references(&wrappers, plugin);

    let type_imports: String = ownership.imports_for(&wrappers);
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
    let wrappers: String = wrappers.replacen(
      Self::COMMANDS_MARKER,
      &format!("{enumeration}\n{}", Self::COMMANDS_MARKER),
      1,
    );
    let (header, body) = wrappers
      .split_once("\n\n")
      .unwrap_or_else(|| panic!("{} has no header", format_path(path)));

    GeneratedOutput::write(
      path,
      &format!(
        "{header}\n\nimport {{ IIpcCommandSignature }} from \"{}\";\n{type_imports}{}\n\n{signatures}",
        imports.signature(),
        body.trim_start()
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

    Self::write_wrappers(
      path,
      plugin,
      &wrappers,
      &format!(
        "import {{ invokeRaw }} from \"{}\";\nimport {{ IIpcCommandSignature }} from \"{}\";",
        imports.raw(),
        imports.signature()
      ),
      &Self::render_enum(
        &format!("The wire name of every raw command of the `{plugin}` plugin."),
        &symbols.raw_command_enum(),
        commands
          .iter()
          .map(|(wire_name, _)| (*wire_name, format!("plugin:{plugin}|{wire_name}"))),
      ),
      &format!(
        "/** Commands answering with raw bytes over IPC, which Specta cannot type. */\nexport const {}",
        symbols.raw_commands()
      ),
      &Self::render_signatures(
        &format!("What each raw command of the `{plugin}` plugin takes and resolves to, keyed by its wire name."),
        &symbols.raw_command_signatures(),
        &symbols.raw_command_enum(),
        &symbols.raw_commands(),
        "IIpcCommandSignature",
        Self::declared_arguments(commands),
      ),
      ownership,
      enumerations,
    );
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

    Self::write_wrappers(
      path,
      plugin,
      &wrappers,
      &format!(
        "import {{ IBulkCall }} from \"{}\";\nimport {{ IIpcBulkRouteSignature }} from \"{}\";",
        imports.bulk(),
        imports.signature()
      ),
      &Self::render_enum(
        &format!("Every bulk route of the `{plugin}` plugin, as the transport serves it."),
        &symbols.bulk_route_enum(),
        routes.iter().map(|(route, _)| (*route, format!("{plugin}/{route}"))),
      ),
      &format!(
        "/** Routes answering with bytes over the loopback transport, which Specta never sees. */\nexport const {}",
        symbols.bulk_routes()
      ),
      &Self::render_signatures(
        &format!("What each bulk route of the `{plugin}` plugin takes, keyed by its route."),
        &symbols.bulk_route_signatures(),
        &symbols.bulk_route_enum(),
        &symbols.bulk_routes(),
        "IIpcBulkRouteSignature",
        Self::declared_arguments(routes),
      ),
      ownership,
      enumerations,
    );
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

  /// Writes a module of wrappers the registry spells by hand: its enum, the wrappers under `declaration`, and their
  /// signatures, importing what they call.
  #[allow(clippy::too_many_arguments)]
  fn write_wrappers(
    path: &Path,
    plugin: &str,
    wrappers: &str,
    import: &str,
    enumeration: &str,
    declaration: &str,
    signatures: &str,
    ownership: &TypeOwnership,
    enumerations: &Enumerations,
  ) {
    // The registry spells these argument types by hand, so they take the same retyping as a Specta signature.
    let wrappers: String = enumerations.rewrite_parameters(wrappers);

    ownership.assert_no_foreign_references(&wrappers, plugin);

    GeneratedOutput::write(
      path,
      &format!(
        "{}\n{import}\n{}\n{enumeration}\n{declaration} = {{\n{wrappers}}};\n\n{signatures}",
        GeneratedOutput::HEADER,
        ownership.imports_for(&wrappers)
      ),
    );
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

  use crate::collected::SpectaCommand;
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

  #[test]
  fn a_typed_module_names_its_commands_by_enum_and_declares_their_signatures() {
    let path: PathBuf = written("fixture.ts", |path, ownership, enumerations| {
      fs::write(
        path,
        "// Auto-generated rust bindings. Do not edit it manually.\n\n\
         import { invoke as __TAURI_INVOKE } from \"@tauri-apps/api/core\";\n\n\
         /** Commands */\n\
         export const commands = {\n  \
         openThing: (sessionId: string, jobId: string) => __TAURI_INVOKE<number>(\"plugin:fixture|open_thing\", { sessionId, jobId }),\n  \
         listThings: () => __TAURI_INVOKE<Array<string>>(\"plugin:fixture|list_things\"),\n\
         };\n\n/* Types */\n",
      )
      .expect("module to be written");

      CommandModuleWriter::finalize(
        path,
        "fixture",
        &[
          SpectaCommand {
            name: String::from("open_thing"),
            arguments: vec![String::from("sessionId"), String::from("jobId")],
          },
          SpectaCommand {
            name: String::from("list_things"),
            arguments: Vec::new(),
          },
        ],
        ownership,
        enumerations,
        &IMPORTS,
      );
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
       export const fixtureCommands = {\n  \
       openThing: (sessionId: string, jobId: string) => __TAURI_INVOKE<number>(EFixtureCommand.OPEN_THING, { sessionId, jobId }),\n  \
       listThings: () => __TAURI_INVOKE<Array<string>>(EFixtureCommand.LIST_THINGS),\n\
       };\n\n\
       /** What each command of the `fixture` plugin takes and resolves to, keyed by its wire name. */\n\
       export interface IFixtureCommandSignatures {\n  \
       [EFixtureCommand.OPEN_THING]: IIpcCommandSignature<typeof fixtureCommands.openThing, [\"sessionId\", \"jobId\"]>;\n  \
       [EFixtureCommand.LIST_THINGS]: IIpcCommandSignature<typeof fixtureCommands.listThings, []>;\n\
       }\n"
    );
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
  #[should_panic(expected = "declares no `export const commands = {`")]
  fn a_module_without_specta_s_command_object_fails() {
    written("renamed.ts", |path, ownership, enumerations| {
      fs::write(
        path,
        "// header\n\nimport { invoke as __TAURI_INVOKE } from \"@tauri-apps/api/core\";\n\n/** Commands */\n\
         export const fixture = {\n};\n",
      )
      .expect("module to be written");

      CommandModuleWriter::finalize(path, "fixture", &[], ownership, enumerations, &IMPORTS);
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
