use clap::{Arg, ArgAction, ArgGroup, ArgMatches, Command, value_parser};
use xrf_output::OutputOptions;
use xrf_particles::{ParticleActionType, ParticleGroup, ParticlesFile};

use super::particle_find_match::ParticleFindMatch;
use super::particle_find_reason::ParticleFindReason;
use super::report::ParticleFindReport;
use crate::commands::particle::list::particle_list_entry::ParticleListEntry;
use crate::commands::particle::particle_names::ParticleNames;
use crate::commands::particle::particles_file_arguments::ParticlesFileArguments;
use crate::core::command::{CommandContext, CommandResult, GenericCommand};

const TEXTURE_ARGUMENT: &str = "texture";
const SHADER_ARGUMENT: &str = "shader";
const ACTION_ARGUMENT: &str = "action";
const EFFECT_ARGUMENT: &str = "effect";

#[derive(Default)]
pub struct FindCommand;

impl GenericCommand for FindCommand {
  fn operation(&self) -> &'static str {
    "find"
  }

  fn declare(&self, command: Command) -> Command {
    ParticlesFileArguments::declare(command.about("Find the effects drawing with a texture or shader or running an action, and the groups playing an effect"))
      .arg(
        Arg::new(TEXTURE_ARGUMENT)
          .help("Texture to find the effects sampling, with or without its extension. Repeat to find several")
          .long(TEXTURE_ARGUMENT)
          .action(ArgAction::Append)
          .value_parser(value_parser!(String)),
      )
      .arg(
        Arg::new(SHADER_ARGUMENT)
          .help("Shader to find the effects drawing with, such as particles\\add. Repeat to find several")
          .long(SHADER_ARGUMENT)
          .action(ArgAction::Append)
          .value_parser(value_parser!(String)),
      )
      .arg(
        Arg::new(ACTION_ARGUMENT)
          .help("Action type to find the effects running, as the unpacked library names it, such as TargetColor. Repeat to find several")
          .long(ACTION_ARGUMENT)
          .action(ArgAction::Append)
          .value_parser(|value: &str| {
            value
              .parse::<ParticleActionType>()
              .map_err(|_| format!("'{value}' is no particle action type"))
          }),
      )
      .arg(
        Arg::new(EFFECT_ARGUMENT)
          .help("Effect to find the groups playing or spawning it. Repeat to find several")
          .long(EFFECT_ARGUMENT)
          .action(ArgAction::Append)
          .value_parser(value_parser!(String)),
      )
      .group(
        ArgGroup::new("query")
          .args([TEXTURE_ARGUMENT, SHADER_ARGUMENT, ACTION_ARGUMENT, EFFECT_ARGUMENT])
          .required(true)
          .multiple(true),
      )
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let output: OutputOptions = context.get_output().clone();
    let file: ParticlesFile = ParticlesFileArguments::open(matches)?;
    let mut report: ParticleFindReport = ParticleFindReport {
      textures: Self::requested(matches, TEXTURE_ARGUMENT),
      shaders: Self::requested(matches, SHADER_ARGUMENT),
      actions: Self::requested(matches, ACTION_ARGUMENT),
      effects: Self::requested(matches, EFFECT_ARGUMENT),
      matches: Vec::new(),
    };

    for effect in &file.effects.effects {
      let entry: ParticleListEntry = ParticleListEntry::of_effect(effect);
      let reasons: Vec<ParticleFindReason> = Self::match_effect(&report, &entry);

      if !reasons.is_empty() {
        report.matches.push(ParticleFindMatch { entry, reasons });
      }
    }

    for group in &file.groups.groups {
      let reasons: Vec<ParticleFindReason> = Self::match_group(&report.effects, group);

      if !reasons.is_empty() {
        report.matches.push(ParticleFindMatch {
          entry: ParticleListEntry::of_group(group),
          reasons,
        });
      }
    }

    for found in &report.matches {
      let reasons: Vec<String> = found.reasons.iter().map(ParticleFindReason::describe).collect();

      xrf_output::info!(output, "{}: {}", found.entry.describe(), reasons.join(", "));
    }

    xrf_output::success!(output, "Found {} effects and groups", report.matches.len());

    context.set_result(|| &report)?;

    Ok(())
  }
}

impl FindCommand {
  fn requested<T: Clone + Send + Sync + 'static>(matches: &ArgMatches, argument: &str) -> Vec<T> {
    matches
      .get_many::<T>(argument)
      .map(|values| values.cloned().collect())
      .unwrap_or_default()
  }

  /// How an effect, as `particle list` describes it, matches the texture, shader and action queries.
  fn match_effect(report: &ParticleFindReport, entry: &ParticleListEntry) -> Vec<ParticleFindReason> {
    let mut reasons: Vec<ParticleFindReason> = Vec::new();
    let ParticleListEntry::Effect {
      shader,
      textures,
      actions,
      ..
    } = entry
    else {
      return reasons;
    };

    for texture in textures {
      if report
        .textures
        .iter()
        .any(|query| ParticleNames::texture_key(query) == ParticleNames::texture_key(texture))
      {
        reasons.push(ParticleFindReason::Texture(texture.clone()));
      }
    }

    if let Some(shader) = shader
      && report
        .shaders
        .iter()
        .any(|query| ParticleNames::name_key(query) == ParticleNames::name_key(shader))
    {
      reasons.push(ParticleFindReason::Shader(shader.clone()));
    }

    for action in &report.actions {
      if actions.contains(action) {
        reasons.push(ParticleFindReason::Action(*action));
      }
    }

    reasons
  }

  /// How a group matches the effect queries: each of its effects named, and each child it spawns.
  fn match_group(queries: &[String], group: &ParticleGroup) -> Vec<ParticleFindReason> {
    let is_queried = |name: &str| {
      queries
        .iter()
        .any(|query| ParticleNames::name_key(query) == ParticleNames::name_key(name))
    };
    let mut reasons: Vec<ParticleFindReason> = Vec::new();
    let mut add = |reason: ParticleFindReason| {
      if !reasons.contains(&reason) {
        reasons.push(reason);
      }
    };

    for effect in &group.effects {
      if is_queried(&effect.name) {
        add(ParticleFindReason::Effect(effect.name.clone()));
      }

      for (child, name) in effect.list_children() {
        if is_queried(name) {
          add(ParticleFindReason::of_child(child, name));
        }
      }
    }

    reasons
  }
}
