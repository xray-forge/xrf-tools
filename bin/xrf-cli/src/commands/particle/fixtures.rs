//! A small particle library the particle commands' tests read, written out as the unpacked library `particle pack`
//! reads, then packed.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use xrf_error::XrfResult;
use xrf_particles::ParticlesFile;
use xrf_spawn::XRayByteOrder;
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

use crate::core::command_testing::run_command_for_result;
use crate::core::generic_command::{CommandResult, GenericCommand};

const HEADER: &str = r"[header]
$type = particles_header
version = 1
";

/// A flame with a frame grid and two textures as `xadd` binds them, a smoke, and a haze drawing only distortion.
const EFFECTS: &str = r"[fx\flame]
$type = particle_effect
version = 1
name = fx\flame
actions_count = 3
max_particles = 8
flags = 32769
time_limit = 2

[fx\flame.sprite]
$type = particle_effect_sprite
shader_name = particles\xadd
texture_name = pfx\pfx_flame.bmp, pfx\pfx_distortion

[fx\flame.frame]
$type = particle_effect_frame
texture_size = 0.25,0.5
reserved = 0,0
frame_dimension_x = 4
frame_count = 8
frame_speed = 30

[fx\flame.action.0]
$type = particle_action
action_flags = 0
action_type = KillOld
age_limit = 5
kill_less_than = 0

[fx\flame.action.1]
$type = particle_action
action_flags = 0
action_type = TargetColor
color = 0.1,0.1,0.1
alpha = 0
scale = 0.5
time_from = 0
time_to = 1

[fx\flame.action.2]
$type = particle_action
action_flags = 0
action_type = Move

[fx\smoke]
$type = particle_effect
version = 1
name = fx\smoke
actions_count = 1
max_particles = 16
flags = 1

[fx\smoke.sprite]
$type = particle_effect_sprite
shader_name = particles\blend
texture_name = pfx\pfx_smoke

[fx\smoke.action.0]
$type = particle_action
action_flags = 0
action_type = Move

[fx\haze]
$type = particle_effect
version = 1
name = fx\haze
actions_count = 0
max_particles = 4
flags = 1

[fx\haze.sprite]
$type = particle_effect_sprite
shader_name = particles\xdistort
texture_name = pfx\pfx_distortion
";

/// A looping campfire playing the flame and the smoke, whose particles each spawn a haze at birth, and a puff of smoke.
const GROUPS: &str = r"[fx\campfire]
$type = particle_group
version = 3
name = fx\campfire
flags = 0
time_limit = 0

[fx\campfire.effect.0]
$type = particle_group_effect
name = fx\flame
on_play_child_name =
on_birth_child_name =
on_dead_child_name =
time_0 = 0
time_1 = 0
flags = 4

[fx\campfire.effect.1]
$type = particle_group_effect
name = fx\smoke
on_play_child_name =
on_birth_child_name = fx\haze
on_dead_child_name =
time_0 = 1
time_1 = 0
flags = 36

[fx\puff]
$type = particle_group
version = 3
name = fx\puff
flags = 0
time_limit = 3

[fx\puff.effect.0]
$type = particle_group_effect
name = fx\smoke
on_play_child_name =
on_birth_child_name =
on_dead_child_name =
time_0 = 0
time_1 = 3
flags = 4
";

/// Write the library under a root of its own, since tests run in parallel, and answer the packed file's path.
pub fn create_library(name: &str) -> XrfResult<PathBuf> {
  let root: PathBuf = build_absolute_generated_test_resource_path(&format!("particle_commands/{name}"));
  let unpacked: PathBuf = root.join("unpacked");
  let packed: PathBuf = root.join("particles.xr");

  if root.exists() {
    fs::remove_dir_all(&root)?;
  }

  fs::create_dir_all(&unpacked)?;
  fs::write(unpacked.join("header.ltx"), HEADER)?;
  fs::write(unpacked.join("effects.ltx"), EFFECTS)?;
  fs::write(unpacked.join("groups.ltx"), GROUPS)?;

  ParticlesFile::import_from_path(&unpacked)?.write_to_path::<XRayByteOrder, _>(&packed)?;

  Ok(packed)
}

/// Run a particle command on a library and answer what it deposited.
pub fn run_for_result<T: GenericCommand>(command: &T, library: &Path, extra: &[&str]) -> CommandResult<Value> {
  let mut arguments: Vec<String> = vec![
    String::from(command.operation()),
    String::from("--path"),
    library.display().to_string(),
    String::from("--silent"),
    String::from("--json"),
  ];

  arguments.extend(extra.iter().map(|it| String::from(*it)));

  Ok(run_command_for_result(command, &arguments)?.expect("Expected the command to deposit a result"))
}
