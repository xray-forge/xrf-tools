use std::collections::HashSet;
use std::path::PathBuf;

use clap::{Arg, ArgGroup, ArgMatches, Command, value_parser};
use xrf_ltx::{LtxDocumentSource, LtxProject};
use xrf_ltx_inspect::{
  LtxDeclaration, LtxDeclarationQuery, LtxDeclarationReader, LtxDeclarationSearch, LtxDeclarationStatement,
  LtxInventory, LtxInventoryReader,
};
use xrf_output::OutputOptions;
use xrf_utils::format_path;

use super::report::LtxFindReport;
use crate::commands::ltx::ltx_installation::LtxInstallation;
use crate::commands::ltx::ltx_project_opening::LtxProjectOpening;
use crate::core::arguments::LtxDialectArgument;
use crate::core::command::{CommandContext, CommandResult, GenericCommand};

const SECTION_ARGUMENT: &str = "section";
const FIELD_ARGUMENT: &str = "field";
const VALUE_ARGUMENT: &str = "value";

#[derive(Default)]
pub struct FindCommand;

impl GenericCommand for FindCommand {
  fn operation(&self) -> &'static str {
    "find"
  }

  fn declare(&self, command: Command) -> Command {
    let command: Command = command
      .about("Find where a section or field is written, declared or patched, across every config a project holds")
      .arg(
        Arg::new("path")
          .help(LtxInstallation::PATH_HELP)
          .short('p')
          .long("path")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new(SECTION_ARGUMENT)
          .help(
            "Section to find, without brackets or operation prefix. Matches the whole name, case-sensitively, except \
             under --dltx, which lowercases section names as the engine does",
          )
          .long(SECTION_ARGUMENT)
          .value_parser(Self::parse_section),
      )
      .arg(
        Arg::new(FIELD_ARGUMENT)
          .help("Field to find, without operation prefix. Matches the whole name, case-sensitively")
          .long(FIELD_ARGUMENT)
          .value_parser(Self::parse_field),
      )
      .arg(
        Arg::new(VALUE_ARGUMENT)
          .help("Text a written value must contain, case-sensitively")
          .long(VALUE_ARGUMENT)
          .value_parser(value_parser!(String)),
      )
      .group(
        ArgGroup::new("query")
          .args([SECTION_ARGUMENT, FIELD_ARGUMENT, VALUE_ARGUMENT])
          .required(true)
          .multiple(true),
      );

    LtxDialectArgument::declare(command)
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let path: &PathBuf = matches
      .get_one::<PathBuf>("path")
      .expect("Expected valid input path to be provided");

    let output: OutputOptions = context.get_output().clone();

    xrf_output::heading!(output, "Find path: {}", format_path(path));

    let project: LtxProject = LtxProjectOpening::open(path, matches, &output)?;
    let source = project.document_source();
    let inventory: LtxInventory = LtxInventoryReader::new(&project, &source as &dyn LtxDocumentSource).read()?;

    let query: LtxDeclarationQuery = LtxDeclarationQuery {
      section: matches.get_one::<String>(SECTION_ARGUMENT).cloned(),
      field: matches.get_one::<String>(FIELD_ARGUMENT).cloned(),
      value: matches.get_one::<String>(VALUE_ARGUMENT).cloned(),
      is_section_case_folded: project.get_dialect().is_folding_section_names(),
    };
    let search: LtxDeclarationSearch =
      LtxDeclarationReader::new(&inventory, &source as &dyn LtxDocumentSource).search(&query)?;

    for declaration in &search.declarations {
      xrf_output::info!(output, "{}", Self::describe(declaration, context.is_verbose()));
    }

    for unreadable in &search.unreadable {
      xrf_output::warning!(
        output,
        "Not searched, {} does not parse at line {}, column {}: {}",
        unreadable.path,
        unreadable.error.line,
        unreadable.error.column,
        unreadable.error.message
      );
    }

    let patches: usize = search.declarations.iter().filter(|it| it.is_patch).count();
    let is_dltx: bool = LtxDialectArgument::is_dltx(matches);

    // The statements are reported either way, since they are written either way; what changes is that standard LTX
    // refuses to resolve the configs that hold them, which a person reading the list would otherwise not learn here.
    if patches > 0 && !is_dltx {
      xrf_output::warning!(
        output,
        "{patches} statement(s) are written as DLTX patches, which standard LTX refuses; rerun with --dltx"
      );
    }

    xrf_output::success!(output, "{}", Self::summarize(&search, patches));

    // A textual match is where something is written, never what it resolves to, so a section query ends by naming
    // the command that answers the second question.
    if let Some(section) = &query.section
      && !search.declarations.is_empty()
    {
      xrf_output::info!(
        output,
        "Resolved values: ltx inspect --path {}{} {section}",
        format_path(path),
        if is_dltx { " --dltx" } else { "" }
      );
    }

    let dialect: &str = project.get_dialect().get_name();

    context.set_result(|| LtxFindReport::new(dialect, &search))?;

    Ok(())
  }
}

impl FindCommand {
  /// Prefixes a header can carry outside its brackets, longest first so `!!` is not read as `!`.
  const SECTION_PREFIXES: [&'static str; 4] = ["!!", "!", "@", ""];

  /// Prefixes the parser takes off every field line, whatever the dialect.
  const FIELD_PREFIXES: [char; 3] = ['!', '>', '<'];

  /// A `--section` query, refused when it is written as a header.
  ///
  /// The parser keeps a header's brackets and operation prefix out of the name it records, so `[wpn_ak74]` or
  /// `![wpn_ak74]` can never match anything, and answering "found 0" would read as nothing declaring the section.
  fn parse_section(value: &str) -> Result<String, String> {
    let is_header = Self::SECTION_PREFIXES.iter().any(|prefix| {
      value
        .strip_prefix(prefix)
        .is_some_and(|rest| rest.starts_with('[') && rest.ends_with(']'))
    });

    if is_header {
      return Err(format!(
        "'{value}' is written as a header; pass the section name alone, without brackets or prefix"
      ));
    }

    Ok(String::from(value))
  }

  /// A `--field` query, refused when it starts with an operation prefix no recorded field name can start with.
  fn parse_field(value: &str) -> Result<String, String> {
    if value.starts_with(Self::FIELD_PREFIXES) {
      return Err(format!(
        "'{value}' starts with an operation prefix; pass the field name alone, and every operation on it is found"
      ));
    }

    Ok(String::from(value))
  }

  /// One statement as a line: where to open, what is written there, and whether it declares or patches.
  fn describe(declaration: &LtxDeclaration, is_verbose: bool) -> String {
    let header: String = declaration
      .section
      .as_ref()
      .map(|section| format!("{}[{section}]", declaration.section_operation))
      .unwrap_or_default();

    let statement: String = match &declaration.statement {
      LtxDeclarationStatement::Section { parents } if parents.is_empty() => header,
      LtxDeclarationStatement::Section { parents } => format!("{header}:{}", parents.join(", ")),
      LtxDeclarationStatement::Field { name, operation, value } => {
        let field: String = match value {
          Some(value) => format!("{operation}{name} = {value}"),
          None => format!("{operation}{name}"),
        };

        // A key written before the first header belongs to no section, and is shown as the bare line it is.
        if header.is_empty() {
          field
        } else {
          format!("{header} {field}")
        }
      }
    };

    let kind: &str = if declaration.is_patch { "patch" } else { "declaration" };

    // Said as list says it: an archived config reads like a loose one, and differs only in that nothing can edit it in
    // place.
    let archived: &str = if declaration.is_physical { "" } else { " [archived]" };

    if is_verbose {
      format!(
        "{}:{} {statement} - {kind}{archived} ({})",
        declaration.path, declaration.line, declaration.source
      )
    } else {
      format!(
        "{}:{} {statement} - {kind}{archived}",
        declaration.path, declaration.line
      )
    }
  }

  /// The counts, so a field written in hundreds of sections says what it amounts to without being read line by line.
  fn summarize(search: &LtxDeclarationSearch, patches: usize) -> String {
    let configs: HashSet<&str> = search
      .declarations
      .iter()
      .map(|declaration| declaration.path.as_str())
      .collect();

    format!(
      "Found {} statement(s) in {} config(s): {} declaration(s), {patches} patch(es)",
      search.declarations.len(),
      configs.len(),
      search.declarations.len() - patches
    )
  }
}
