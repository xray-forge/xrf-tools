use clap::{Arg, ArgMatches, Command, value_parser};
use xrf_error::XrfError;
use xrf_output::OutputOptions;
use xrf_particles::{
  ParticleActionType, ParticleEffect, ParticleEffectFlags, ParticleGroup, ParticleGroupChild, ParticleGroupEffectFlags,
  ParticlesFile,
};
use xrf_utils::format_path;

use super::report::ParticleInspectReport;
use crate::commands::particle::particle_names::to_name_key;
use crate::commands::particle::particles_file_open::{
  ParticlesFileArguments, open_particles_file, requested_particles_path,
};
use crate::core::command_context::CommandContext;
use crate::core::generic_command::{CommandResult, GenericCommand};

const NAME_ARGUMENT: &str = "name";

/// How many names a missing one is answered with, at most.
const SUGGESTIONS: usize = 10;

#[derive(Default)]
pub struct InspectCommand;

impl GenericCommand for InspectCommand {
  fn operation(&self) -> &'static str {
    "inspect"
  }

  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Explain one effect or group: its sprite, frames and actions, or the effects it plays and spawns")
      .with_particles_file()
      .arg(
        Arg::new(NAME_ARGUMENT)
          .help("Name of the effect or group, in any case and with either slash, such as explosions\\campfire")
          .required(true)
          .value_parser(value_parser!(String)),
      )
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let output: OutputOptions = context.get_output().clone();
    let file: ParticlesFile = open_particles_file(matches)?;
    let name: &String = matches
      .get_one::<String>(NAME_ARGUMENT)
      .expect("Expected the required name");
    let is_named = |candidate: &str| to_name_key(candidate) == to_name_key(name);
    let report: ParticleInspectReport<'_> = ParticleInspectReport {
      effect: file.effects.effects.iter().find(|it| is_named(&it.name)),
      group: file.groups.groups.iter().find(|it| is_named(&it.name)),
    };

    if report.effect.is_none() && report.group.is_none() {
      return Err(
        XrfError::new_not_found_error(format!(
          "No effect or group '{name}' in {}{}",
          format_path(requested_particles_path(matches)),
          Self::describe_suggestions(&file, name)
        ))
        .into(),
      );
    }

    for line in report.effect.map(Self::describe_effect).unwrap_or_default() {
      xrf_output::info!(output, "{line}");
    }

    for line in report.group.map(Self::describe_group).unwrap_or_default() {
      xrf_output::info!(output, "{line}");
    }

    context.set_result(|| &report)?;

    Ok(())
  }
}

impl InspectCommand {
  /// An effect as lines: its sprite, particles and flags, frames and collision where it has them, then its actions.
  fn describe_effect(effect: &ParticleEffect) -> Vec<String> {
    let flags: Vec<&str> = ParticleEffectFlags(effect.flags).list_names();
    let mut lines: Vec<String> = vec![
      format!("effect {}", effect.name),
      format!(
        "  sprite: {}, {}",
        effect.sprite.shader_name, effect.sprite.texture_name
      ),
      format!("  max particles: {}", effect.max_particles),
      format!("  flags: {}", flags.join(", ")),
      format!(
        "  time limit: {}",
        effect
          .time_limit
          .filter(|it| *it > 0.0)
          .map_or(String::from("none, loops"), |it| format!("{it} s"))
      ),
    ];

    if let Some(frame) = &effect.frame {
      lines.push(format!(
        "  frames: {} in rows of {}, at {} a second",
        frame.frame_count, frame.frame_dimension_x, frame.frame_speed
      ));
    }

    if let Some(collision) = &effect.collision {
      lines.push(format!(
        "  collision: friction {}, resilience {}, cutoff {}",
        1.0 - collision.collide_one_minus_friction,
        collision.collide_resilience,
        collision.collide_sqr_cutoff.sqrt()
      ));
    }

    lines.push(format!("  actions: {}", effect.actions.len()));
    lines.extend(
      effect
        .actions
        .iter()
        .enumerate()
        .map(|(index, action)| format!("    {index}. {}", ParticleActionType::get_action_type(action))),
    );

    lines
  }

  /// A group as lines: how long it plays, then each effect with its window, flags and children.
  fn describe_group(group: &ParticleGroup) -> Vec<String> {
    let mut lines: Vec<String> = vec![
      format!("group {}", group.name),
      format!(
        "  time limit: {}",
        if group.time_limit > 0.0 {
          format!("{} s", group.time_limit)
        } else {
          String::from("none, loops")
        }
      ),
      format!("  effects: {}", group.effects.len()),
    ];

    for (index, effect) in group.effects.iter().enumerate() {
      let flags: Vec<&str> = ParticleGroupEffectFlags(effect.flags).list_names();

      lines.push(format!(
        "    {index}. {} from {} s to {} s ({})",
        effect.name,
        effect.time_0,
        effect.time_1,
        flags.join(", ")
      ));

      for (child, name) in effect.list_children() {
        let when: &str = match child {
          ParticleGroupChild::Play => "on play",
          ParticleGroupChild::Birth => "on birth",
          ParticleGroupChild::Death => "on death",
        };

        lines.push(format!("       {when}: {name}"));
      }
    }

    lines
  }

  /// The names holding the one asked for, as a clause for the not-found error, or nothing where none does.
  fn describe_suggestions(file: &ParticlesFile, name: &str) -> String {
    let query: String = to_name_key(name);
    let effects = file.effects.effects.iter().map(|it| it.name.as_str());
    let groups = file.groups.groups.iter().map(|it| it.name.as_str());
    let similar: Vec<&str> = effects
      .chain(groups)
      .filter(|it| to_name_key(it).contains(&query))
      .take(SUGGESTIONS)
      .collect();

    if similar.is_empty() {
      String::new()
    } else {
      format!("; names holding it: {}", similar.join(", "))
    }
  }
}
