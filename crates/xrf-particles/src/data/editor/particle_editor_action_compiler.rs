use xrf_error::{XrfError, XrfResult};
use xrf_math::Vector3d;

use crate::data::actions::particle_action_avoid::ParticleActionAvoid;
use crate::data::actions::particle_action_bounce::ParticleActionBounce;
use crate::data::actions::particle_action_copy_vertex::ParticleActionCopyVertex;
use crate::data::actions::particle_action_damping::ParticleActionDamping;
use crate::data::actions::particle_action_explosion::ParticleActionExplosion;
use crate::data::actions::particle_action_follow::ParticleActionFollow;
use crate::data::actions::particle_action_gravitate::ParticleActionGravitate;
use crate::data::actions::particle_action_gravity::ParticleActionGravity;
use crate::data::actions::particle_action_jet::ParticleActionJet;
use crate::data::actions::particle_action_kill_old::ParticleActionKillOld;
use crate::data::actions::particle_action_match_velocity::ParticleActionMatchVelocity;
use crate::data::actions::particle_action_move::ParticleActionMove;
use crate::data::actions::particle_action_orbit_line::ParticleActionOrbitLine;
use crate::data::actions::particle_action_orbit_point::ParticleActionOrbitPoint;
use crate::data::actions::particle_action_random_acceleration::ParticleActionRandomAcceleration;
use crate::data::actions::particle_action_random_displace::ParticleActionRandomDisplace;
use crate::data::actions::particle_action_random_velocity::ParticleActionRandomVelocity;
use crate::data::actions::particle_action_restore::ParticleActionRestore;
use crate::data::actions::particle_action_scatter::ParticleActionScatter;
use crate::data::actions::particle_action_sink::ParticleActionSink;
use crate::data::actions::particle_action_sink_velocity::ParticleActionSinkVelocity;
use crate::data::actions::particle_action_source::ParticleActionSource;
use crate::data::actions::particle_action_speed_limit::ParticleActionSpeedLimit;
use crate::data::actions::particle_action_target_color::ParticleActionTargetColor;
use crate::data::actions::particle_action_target_rotate::ParticleActionTargetRotate;
use crate::data::actions::particle_action_target_size::ParticleActionTargetSize;
use crate::data::actions::particle_action_target_velocity::ParticleActionTargetVelocity;
use crate::data::actions::particle_action_turbulence::ParticleActionTurbulence;
use crate::data::actions::particle_action_vortex::ParticleActionVortex;
use crate::data::editor::particle_editor_action::ParticleEditorAction;
use crate::data::particle_action::ParticleAction;
use crate::data::particle_action_type::ParticleActionType;
use crate::data::particle_domain::ParticleDomain;
use crate::simulation::domain::engine_vector::EngineVector;

/// Turns an editor action into the action the engine plays, as each `EPA*::Compile` calls its `p*` function
/// (`editors/ParticleEditor/ParticleEffectActionsImpl.cpp`).
pub(crate) struct ParticleEditorActionCompiler;

impl ParticleEditorActionCompiler {
  /// `EPS_L`, which an explosion's softening falls back to where it is negative.
  const SOFTENING: f32 = 0.001;

  /// # Errors
  ///
  /// Returns an error when the action lacks a property its kind needs, or is of a kind nothing compiles.
  pub fn compile(action: &ParticleEditorAction) -> XrfResult<ParticleAction> {
    let flags: u32 = Self::read_rotate_flag(action)?;

    Ok(match action.action_type {
      ParticleActionType::Avoid => ParticleAction::Avoid(Box::new(ParticleActionAvoid {
        action_flags: flags,
        action_type: ParticleActionType::Avoid,
        position: Self::read_domain(action, "Position")?,
        look_ahead: action.get_float("Look Ahead")?,
        magnitude: action.get_float("Magnitude")?,
        epsilon: action.get_float("Epsilon")?,
      })),
      ParticleActionType::Bounce => ParticleAction::Bounce(Box::new(ParticleActionBounce {
        action_flags: flags,
        action_type: ParticleActionType::Bounce,
        position: Self::read_domain(action, "Position")?,
        one_minus_friction: 1.0 - action.get_float("Friction")?,
        resilience: action.get_float("Resilience")?,
        cutoff_sqr: Self::square(action.get_float("Cutoff")?),
      })),
      ParticleActionType::CopyVertex => ParticleAction::CopyVertex(Box::new(ParticleActionCopyVertex {
        action_flags: flags,
        action_type: ParticleActionType::CopyVertex,
        copy_position: u32::from(action.get_bool("Copy Position")?),
      })),
      ParticleActionType::Damping => ParticleAction::Damping(Box::new(ParticleActionDamping {
        action_flags: flags,
        action_type: ParticleActionType::Damping,
        damping: action.get_vector("Damping")?.clone(),
        v_low_sqr: Self::square(action.get_float("V Low")?),
        v_high_sqr: Self::square(action.get_float("V High")?),
      })),
      ParticleActionType::Explosion => ParticleAction::Explosion(Box::new(ParticleActionExplosion {
        action_flags: flags,
        action_type: ParticleActionType::Explosion,
        center: action.get_vector("Center")?.clone(),
        velocity: action.get_float("Velocity")?,
        magnitude: action.get_float("Magnitude")?,
        st_dev: action.get_float("Standart Dev")?,
        age: action.get_float("Age")?,
        epsilon: Some(action.get_float("Epsilon")?)
          .filter(|epsilon| *epsilon >= 0.0)
          .unwrap_or(Self::SOFTENING),
      })),
      ParticleActionType::Follow => ParticleAction::Follow(Box::new(ParticleActionFollow {
        action_flags: flags,
        action_type: ParticleActionType::Follow,
        magnitude: action.get_float("Magnitude")?,
        epsilon: action.get_float("Epsilon")?,
        max_radius: action.get_float("Max Radius")?,
      })),
      ParticleActionType::Gravitate => ParticleAction::Gravitate(Box::new(ParticleActionGravitate {
        action_flags: flags,
        action_type: ParticleActionType::Gravitate,
        magnitude: action.get_float("Magnitude")?,
        epsilon: action.get_float("Epsilon")?,
        max_radius: action.get_float("Max Radius")?,
      })),
      ParticleActionType::Gravity => ParticleAction::Gravity(Box::new(ParticleActionGravity {
        action_flags: flags,
        action_type: ParticleActionType::Gravity,
        direction: action.get_vector("Direction")?.clone(),
      })),
      ParticleActionType::Jet => ParticleAction::Jet(Box::new(ParticleActionJet {
        action_flags: flags,
        action_type: ParticleActionType::Jet,
        center: action.get_vector("Center")?.clone(),
        acc: Self::read_domain(action, "Accelerate")?,
        magnitude: action.get_float("Magnitude")?,
        epsilon: action.get_float("Epsilon")?,
        max_radius: action.get_float("Max Radius")?,
      })),
      ParticleActionType::KillOld => ParticleAction::KillOld(Box::new(ParticleActionKillOld {
        action_flags: flags,
        action_type: ParticleActionType::KillOld,
        age_limit: action.get_float("Age Limit")?,
        kill_less_than: u32::from(action.get_bool("Kill Less Than")?),
      })),
      ParticleActionType::MatchVelocity => ParticleAction::MatchVelocity(Box::new(ParticleActionMatchVelocity {
        action_flags: flags,
        action_type: ParticleActionType::MatchVelocity,
        magnitude: action.get_float("Magnitude")?,
        epsilon: action.get_float("Epsilon")?,
        max_radius: action.get_float("Max Radius")?,
      })),
      ParticleActionType::Move => ParticleAction::Move(Box::new(ParticleActionMove {
        action_flags: flags,
        action_type: ParticleActionType::Move,
      })),
      ParticleActionType::OrbitLine => ParticleAction::OrbitLine(Box::new(ParticleActionOrbitLine {
        action_flags: flags,
        action_type: ParticleActionType::OrbitLine,
        position: action.get_vector("Position")?.clone(),
        axis: action.get_vector("Axis")?.clone().normalize_safe(),
        magnitude: action.get_float("Magnitude")?,
        epsilon: action.get_float("Epsilon")?,
        max_radius: action.get_float("Max Radius")?,
      })),
      ParticleActionType::OrbitPoint => ParticleAction::OrbitPoint(Box::new(ParticleActionOrbitPoint {
        action_flags: flags,
        action_type: ParticleActionType::OrbitPoint,
        center: action.get_vector("Center")?.clone(),
        magnitude: action.get_float("Magnitude")?,
        epsilon: action.get_float("Epsilon")?,
        max_radius: action.get_float("Max Radius")?,
      })),
      ParticleActionType::RandomAccel => ParticleAction::RandomAccel(Box::new(ParticleActionRandomAcceleration {
        action_flags: flags,
        action_type: ParticleActionType::RandomAccel,
        gen_acc: Self::read_domain(action, "Accelerate")?,
      })),
      ParticleActionType::RandomDisplace => ParticleAction::RandomDisplace(Box::new(ParticleActionRandomDisplace {
        action_flags: flags,
        action_type: ParticleActionType::RandomDisplace,
        gen_disp: Self::read_domain(action, "Displace")?,
      })),
      ParticleActionType::RandomVelocity => ParticleAction::RandomVelocity(Box::new(ParticleActionRandomVelocity {
        action_flags: flags,
        action_type: ParticleActionType::RandomVelocity,
        gen_vel: Self::read_domain(action, "Velocity")?,
      })),
      ParticleActionType::Restore => ParticleAction::Restore(Box::new(ParticleActionRestore {
        action_flags: flags,
        action_type: ParticleActionType::Restore,
        time_left: action.get_float("Time")?,
      })),
      ParticleActionType::Scatter => ParticleAction::Scatter(Box::new(ParticleActionScatter {
        action_flags: flags,
        action_type: ParticleActionType::Scatter,
        center: action.get_vector("Center")?.clone(),
        magnitude: action.get_float("Magnitude")?,
        epsilon: action.get_float("Epsilon")?,
        max_radius: action.get_float("Max Radius")?,
      })),
      ParticleActionType::Sink => ParticleAction::Sink(Box::new(ParticleActionSink {
        action_flags: flags,
        action_type: ParticleActionType::Sink,
        kill_inside: u32::from(action.get_bool("Kill Inside")?),
        position: Self::read_domain(action, "Domain")?,
      })),
      ParticleActionType::SinkVelocity => ParticleAction::SinkVelocity(Box::new(ParticleActionSinkVelocity {
        action_flags: flags,
        action_type: ParticleActionType::SinkVelocity,
        kill_inside: u32::from(action.get_bool("Kill Inside")?),
        velocity: Self::read_domain(action, "Domain")?,
      })),
      ParticleActionType::Source => ParticleAction::Source(Box::new(ParticleActionSource {
        action_flags: flags
          | ParticleActionSource::VERTEX_B_TRACKS
          | if action.get_bool("Single Size")? {
            ParticleActionSource::SINGLE_SIZE
          } else {
            0
          },
        action_type: ParticleActionType::Source,
        position: Self::read_domain(action, "Domain")?,
        velocity: Self::read_domain(action, "Velocity")?,
        rot: Self::read_domain(action, "Rotation")?,
        size: Self::read_domain(action, "Size")?,
        color: Self::read_domain(action, "Color")?,
        alpha: action.get_float("Color\\Alpha")?,
        particle_rate: action.get_float("Rate")?,
        age: action.get_float("Starting Age")?,
        age_sigma: action.get_float("Age Sigma")?,
        parent_vel: Vector3d::ZERO,
        parent_motion: action.get_float("Parent Motion")?,
      })),
      ParticleActionType::SpeedLimit => ParticleAction::SpeedLimit(Box::new(ParticleActionSpeedLimit {
        action_flags: flags,
        action_type: ParticleActionType::SpeedLimit,
        min_speed: action.get_float("Min Speed")?,
        max_speed: action.get_float("Max Speed")?,
      })),
      // An action saved before `TimeFrom` and `TimeTo` existed compiles at the span a new one starts with: all of it.
      ParticleActionType::TargetColor => ParticleAction::TargetColor(Box::new(ParticleActionTargetColor {
        action_flags: flags,
        action_type: ParticleActionType::TargetColor,
        color: action.get_vector("Color")?.clone(),
        alpha: action.get_float("Alpha")?,
        scale: action.get_float("Scale")?,
        time_from: action.get_float("TimeFrom").unwrap_or(0.0),
        time_to: action.get_float("TimeTo").unwrap_or(1.0),
      })),
      ParticleActionType::TargetSize => ParticleAction::TargetSize(Box::new(ParticleActionTargetSize {
        action_flags: flags,
        action_type: ParticleActionType::TargetSize,
        size: action.get_vector("Size")?.clone(),
        scale: action.get_vector("Scale")?.clone(),
      })),
      // The SDK edits both rotations as one action and always compiles the plain one.
      ParticleActionType::TargetRotate | ParticleActionType::TargetRotateD => {
        ParticleAction::TargetRotate(Box::new(ParticleActionTargetRotate {
          action_flags: flags,
          action_type: ParticleActionType::TargetRotate,
          rot: action.get_vector("Rotation")?.clone(),
          scale: action.get_float("Scale")?,
        }))
      }
      ParticleActionType::TargetVelocity | ParticleActionType::TargetVelocityD => {
        ParticleAction::TargetVelocity(Box::new(ParticleActionTargetVelocity {
          action_flags: flags,
          action_type: ParticleActionType::TargetVelocity,
          velocity: action.get_vector("Velocity")?.clone(),
          scale: action.get_float("Scale")?,
        }))
      }
      ParticleActionType::Vortex => ParticleAction::Vortex(Box::new(ParticleActionVortex {
        action_flags: flags,
        action_type: ParticleActionType::Vortex,
        center: action.get_vector("Center")?.clone(),
        axis: action.get_vector("Axis")?.clone().normalize_safe(),
        magnitude: action.get_float("Magnitude")?,
        epsilon: action.get_float("Epsilon")?,
        max_radius: action.get_float("Max Radius")?,
      })),
      ParticleActionType::Turbulence => ParticleAction::Turbulence(Box::new(ParticleActionTurbulence {
        action_flags: flags,
        action_type: ParticleActionType::Turbulence,
        frequency: action.get_float("Frequency")?,
        octaves: action.get_int("Octaves")?,
        magnitude: action.get_float("Magnitude")?,
        epsilon: action.get_float("Delta")?,
        offset: action.get_vector("Movement")?.clone(),
      })),
      ParticleActionType::CallActionList | ParticleActionType::Unknown => {
        return Err(XrfError::new_invalid_error(format!(
          "Particle editor action '{}' is of type {}, which nothing compiles",
          action.name, action.action_type
        )));
      }
    })
  }

  /// `ALLOW_ROTATE` where the action has an `Allow Rotate` property that is set; actions without one never turn.
  fn read_rotate_flag(action: &ParticleEditorAction) -> XrfResult<u32> {
    Ok(match action.get("Allow Rotate") {
      Some(_) if action.get_bool("Allow Rotate")? => ParticleAction::ALLOW_ROTATE,
      _ => 0,
    })
  }

  fn read_domain(action: &ParticleEditorAction, name: &str) -> XrfResult<ParticleDomain> {
    Ok(action.get_domain(name)?.compile())
  }

  /// `_sqr`, which the SDK keeps cutoff and damping speeds as.
  fn square(value: f32) -> f32 {
    value * value
  }
}
