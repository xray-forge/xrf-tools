use clap::{Arg, ArgMatches, Command, value_parser};
use xrf_output::OutputOptions;
use xrf_particles::ParticlesFile;

use super::particle_list_entry::ParticleListEntry;
use super::report::ParticleListReport;
use crate::commands::particle::particle_names::ParticleNames;
use crate::commands::particle::particles_file_arguments::ParticlesFileArguments;
use crate::core::command::{CommandContext, CommandResult, GenericCommand};

const KIND_ARGUMENT: &str = "kind";
const NAME_ARGUMENT: &str = "name";

const KIND_EFFECT: &str = "effect";
const KIND_GROUP: &str = "group";

#[derive(Default)]
pub struct ListCommand;

impl GenericCommand for ListCommand {
  fn operation(&self) -> &'static str {
    "list"
  }

  fn declare(&self, command: Command) -> Command {
    ParticlesFileArguments::declare(command.about("List the effects and groups a particle library holds, with what each draws and plays"))
      .arg(
        Arg::new(KIND_ARGUMENT)
          .help("List only effects or only groups")
          .long(KIND_ARGUMENT)
          .value_parser([KIND_EFFECT, KIND_GROUP]),
      )
      .arg(
        Arg::new(NAME_ARGUMENT)
          .help("List only names containing this text, in any case and with either slash, such as campfire or explosions\\effects")
          .long(NAME_ARGUMENT)
          .value_parser(value_parser!(String)),
      )
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let output: OutputOptions = context.get_output().clone();
    let file: ParticlesFile = ParticlesFileArguments::open(matches)?;
    let kind: Option<&str> = matches.get_one::<String>(KIND_ARGUMENT).map(String::as_str);
    let name: Option<String> = matches
      .get_one::<String>(NAME_ARGUMENT)
      .map(|it| ParticleNames::name_key(it));
    let is_named = |entry: &ParticleListEntry| {
      name
        .as_deref()
        .is_none_or(|name| ParticleNames::name_key(entry.get_name()).contains(name))
    };

    let effects = file
      .effects
      .effects
      .iter()
      .filter(|_| kind != Some(KIND_GROUP))
      .map(ParticleListEntry::of_effect);
    let groups = file
      .groups
      .groups
      .iter()
      .filter(|_| kind != Some(KIND_EFFECT))
      .map(ParticleListEntry::of_group);
    let report: ParticleListReport = ParticleListReport::new(effects.chain(groups).filter(is_named).collect());

    for entry in report.entries() {
      xrf_output::info!(output, "{}", entry.describe());
    }

    let effects: usize = report
      .entries()
      .iter()
      .filter(|entry| matches!(entry, ParticleListEntry::Effect { .. }))
      .count();

    xrf_output::success!(
      output,
      "Listed {effects} effects and {} groups",
      report.entries().len() - effects
    );

    context.set_result(|| &report)?;

    Ok(())
  }
}
