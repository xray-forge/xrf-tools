use std::collections::HashSet;
use std::path::PathBuf;

use clap::{Arg, ArgMatches, Command, value_parser};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{LtxDocumentSource, LtxProject, LtxResolution, fold_section_name};
use xrf_ltx_inspect::{
  LtxDeclarationQuery, LtxDeclarationReader, LtxInventory, LtxInventoryReader, LtxResolvedDiagnostic, LtxResolvedField,
  LtxResolvedFieldOrigin, LtxResolvedSection, LtxRootReader,
};
use xrf_output::OutputOptions;
use xrf_utils::format_path;
use xrf_vfs::XrayLogicalPath;

use super::report::LtxInspectReport;
use crate::commands::ltx::ltx_installation::LtxInstallation;
use crate::commands::ltx::ltx_project_opening::LtxProjectOpening;
use crate::core::arguments::{InputArgument, LtxDialectArgument};
use crate::core::command::{CommandContext, CommandInput, CommandResult, GenericCommand};

/// How wide the value column may grow before a value simply takes the room it needs.
///
/// A condlist runs to hundreds of characters, and padding every other row to it would put the origins past the edge of
/// any terminal.
const VALUE_COLUMN_CAP: usize = 44;

#[derive(Default)]
pub(crate) struct InspectCommand;

impl GenericCommand for InspectCommand {
  fn operation(&self) -> &'static str {
    "inspect"
  }

  fn input(&self) -> CommandInput {
    LtxInstallation::input()
  }

  fn declare(&self, command: Command) -> Command {
    let command: Command = command
      .about("Explain one resolved section: its fields, and where each value is written")
      .arg(
        Arg::new("section")
          .help("Name of the section to explain, without its brackets")
          .long("section")
          .required(true)
          .value_parser(value_parser!(String)),
      )
      .arg(
        Arg::new("entry")
          .help("Entry point to resolve, when more than one declares the section")
          .short('e')
          .long("entry")
          .value_parser(value_parser!(String)),
      );

    LtxDialectArgument::declare(command)
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let path: &PathBuf = InputArgument::of(matches);
    let section: &String = matches
      .get_one::<String>("section")
      .expect("Expected a section name to be provided");

    let output: OutputOptions = context.get_output().clone();

    xrf_output::heading!(output, "Inspect path: {}", format_path(path));

    let project: LtxProject = LtxProjectOpening::open(path, matches, &output)?;
    // Lowercased once, as the dialect stores section names, so the comparisons below match.
    let section: &String = &if project.get_dialect().is_folding_section_names() {
      fold_section_name(section).into_owned()
    } else {
      section.clone()
    };
    let entry: XrayLogicalPath = Self::locate(&project, section, matches.get_one::<String>("entry"), &output)?;

    let resolution: LtxResolution = project.resolve_explained(&entry)?;
    let source = project.document_source();

    let dialect: &str = project.get_dialect().get_name();
    let reader: LtxRootReader<'_> =
      LtxRootReader::new(entry.as_str(), dialect, &resolution, &source as &dyn LtxDocumentSource);

    let Some(resolved) = reader.read_sections(&[section.as_str()])?.pop() else {
      return Err(
        XrfError::new_read_error(format!(
          "Section '[{section}]' resolved from '{}' but could not be read back",
          entry.as_str()
        ))
        .into(),
      );
    };

    let diagnostics: Vec<LtxResolvedDiagnostic> = resolution
      .diagnostics
      .iter()
      .filter(|diagnostic| diagnostic.section == *section)
      .map(LtxResolvedDiagnostic::from)
      .collect();

    Self::render(&resolved, &diagnostics, dialect, &output);

    context.set_result(|| LtxInspectReport::new(dialect, resolved, diagnostics))?;

    Ok(())
  }
}

impl InspectCommand {
  /// The entry point whose resolution holds `section`.
  ///
  /// Searches the project's cached plain resolutions, so only the answering one pays for provenance; `--entry` skips
  /// the search. Every candidate is tried, in the project's order, because a section name is not unique across a tree
  /// (Anomaly's `configs\plugins\actor_effects.ltx` declares an unrelated `[wpn_ak74]`): the first answers, and the
  /// others are named in a warning so a person can ask again with `--entry`.
  ///
  /// # Errors
  ///
  /// Returns an error when `--entry` is not an entry point of this project, or when no entry point holds the section.
  fn locate(
    project: &LtxProject,
    section: &str,
    requested: Option<&String>,
    output: &OutputOptions,
  ) -> XrfResult<XrayLogicalPath> {
    if let Some(requested) = requested {
      let requested: XrayLogicalPath = XrayLogicalPath::new(requested)?;

      if !project.ltx_file_entries.contains(&requested) {
        return Err(XrfError::new_read_error(format!(
          "Cannot inspect '{}', it is not an entry point of this project",
          requested.as_str()
        )));
      }

      return Ok(requested);
    }

    let holding: Option<HashSet<String>> = Self::list_holding_entry_points(project, section)?;
    let mut declaring: Vec<&XrayLogicalPath> = Vec::new();
    let mut searched: usize = 0;

    for entry in &project.ltx_file_entries {
      if LtxProject::is_ltx_scheme_path(entry) {
        continue;
      }

      if holding
        .as_ref()
        .is_some_and(|holding| !holding.contains(entry.as_str()))
      {
        continue;
      }

      searched += 1;

      // Not retained: where nothing bounds the search this walks every entry point of the tree, and holding each
      // resolution to answer one question about it would end the search holding the whole project resolved.
      let Ok(resolved) = project.read_resolution(entry) else {
        continue;
      };

      if resolved.ltx.section(section).is_some() {
        declaring.push(entry);
      }
    }

    let Some(first) = declaring.first() else {
      return Err(XrfError::new_read_error(format!(
        "No entry point of this project declares section '[{section}]', resolved {searched} that could"
      )));
    };

    if declaring.len() > 1 {
      xrf_output::warning!(
        output,
        "{} entry points declare '[{section}]'; showing the first. Others: {}",
        declaring.len(),
        declaring
          .iter()
          .skip(1)
          .map(|entry| entry.as_str())
          .collect::<Vec<&str>>()
          .join(", ")
      );
    }

    Ok((*first).clone())
  }

  /// The entry points whose resolution can hold `section`, or `None` when nothing bounds them and all must be tried.
  ///
  /// Read without resolving, from the headers naming the section or naming it as a parent; an attachment that could add
  /// it leaves nothing bound.
  ///
  /// # Errors
  ///
  /// Returns an error when the project's configs cannot be listed or read.
  fn list_holding_entry_points(project: &LtxProject, section: &str) -> XrfResult<Option<HashSet<String>>> {
    let source = project.document_source();
    let inventory: LtxInventory = LtxInventoryReader::new(project, &source as &dyn LtxDocumentSource).read()?;
    let query: LtxDeclarationQuery = LtxDeclarationQuery {
      section: Some(String::from(section)),
      is_section_case_folded: project.get_dialect().is_folding_section_names(),
      ..LtxDeclarationQuery::default()
    };

    Ok(
      LtxDeclarationReader::new(&inventory, &source as &dyn LtxDocumentSource)
        .list_holding_entry_points(&query)?
        .map(|holding| holding.into_iter().collect()),
    )
  }

  /// Prints the section the way a person reads it: the value first, then why it says that.
  fn render(
    section: &LtxResolvedSection,
    diagnostics: &[LtxResolvedDiagnostic],
    dialect: &str,
    output: &OutputOptions,
  ) {
    xrf_output::info!(output, "[{}] resolved from {} ({dialect})", section.name, section.entry);

    if let Some(origin) = &section.origin {
      xrf_output::info!(output, "  declared in {origin}");
    }

    if !section.parents.is_empty() {
      xrf_output::info!(output, "  inherits {}", section.parents.join(", "));
    }

    // Both columns padded, so the origins read as a column rather than as prose trailing each value. The value pad is
    // capped: one long condlist would otherwise push every origin off the far side of a terminal, and a value past the
    // cap simply takes the width it needs.
    let keys: usize = section.fields.iter().map(|field| field.key.len()).max().unwrap_or(0);
    let values: usize = section
      .fields
      .iter()
      .map(|field| field.value.len())
      .filter(|width| *width <= VALUE_COLUMN_CAP)
      .max()
      .unwrap_or(0);

    for field in &section.fields {
      xrf_output::info!(
        output,
        "  {:keys$} = {:values$}  {}",
        field.key,
        field.value,
        Self::describe_origin(field)
      );
    }

    for diagnostic in diagnostics {
      match &diagnostic.engine_behaviour {
        Some(behaviour) => xrf_output::warning!(output, "  ! {} ({behaviour})", diagnostic.message),
        None => xrf_output::warning!(output, "  ! {}", diagnostic.message),
      }
    }

    xrf_output::success!(
      output,
      "{} field(s), {} diagnostic(s)",
      section.fields.len(),
      diagnostics.len()
    );
  }

  /// Why one field holds what it holds, in the fewest words that name the file to open.
  fn describe_origin(field: &LtxResolvedField) -> String {
    match &field.origin {
      LtxResolvedFieldOrigin::Declared { file: Some(file) } => format!("written here, in {file}"),
      LtxResolvedFieldOrigin::Declared { file: None } => String::from("written here"),
      LtxResolvedFieldOrigin::Inherited {
        section,
        file: Some(file),
      } => format!("inherited from [{section}] in {file}"),
      LtxResolvedFieldOrigin::Inherited { section, file: None } => format!("inherited from [{section}]"),
      LtxResolvedFieldOrigin::Loaded { file, depth, operation } if operation.is_empty() => {
        format!("set by {file} (depth {depth})")
      }
      LtxResolvedFieldOrigin::Loaded { file, depth, operation } => {
        format!("set by {file} ('{operation}', depth {depth})")
      }
      // Only reachable if a caller resolved without asking, which this command never does. Said rather than shown as a
      // blank, because a viewer inventing "written here" for an unrecorded field is the one wrong answer here.
      LtxResolvedFieldOrigin::Unrecorded => String::from("origin not recorded"),
    }
  }
}
