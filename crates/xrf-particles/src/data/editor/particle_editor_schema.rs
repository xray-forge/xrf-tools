use xrf_error::{XrfError, XrfResult};
use xrf_math::Vector3d;

use crate::data::editor::particle_editor_domain::ParticleEditorDomain;
use crate::data::editor::particle_editor_property::ParticleEditorProperty;
use crate::data::editor::particle_editor_value::ParticleEditorValue;
use crate::data::particle_action_type::ParticleActionType;

/// The properties each kind of editor action has, with the values a new one starts from, as the SDK's `EPA*`
/// constructors append them (`editors/ParticleEditor/ParticleEffectActionsImpl.cpp`).
pub struct ParticleEditorSchema;

impl ParticleEditorSchema {
  /// `PARTICLE_ACTION_VERSION` of the Clear Sky and Call of Pripyat SDKs, which every new action is written at.
  pub const VERSION: u32 = 1;

  /// The Shadow of Chernobyl SDK's version, whose target colour has no `TimeFrom` and `TimeTo`.
  pub const SHADOW_OF_CHERNOBYL_VERSION: u32 = 0;

  /// `P_MAXFLOAT`, the SDK's stand-in for an unbounded distance or speed.
  const UNBOUNDED: f32 = 1.0e16;

  /// `EPS_L`, the softening most forces start with.
  const SOFTENING: f32 = 0.001;

  /// The name the SDK gives a kind of action (`actionType`), which a new action is also named.
  ///
  /// # Errors
  ///
  /// Returns an error for a type no editor action is made of.
  pub fn get_type_name(action_type: ParticleActionType) -> XrfResult<&'static str> {
    Ok(match action_type {
      ParticleActionType::Avoid => "Avoid",
      ParticleActionType::Bounce => "Bounce",
      ParticleActionType::CopyVertex => "CopyVertexB",
      ParticleActionType::Damping => "Damping",
      ParticleActionType::Explosion => "Explosion",
      ParticleActionType::Follow => "Follow",
      ParticleActionType::Gravitate => "Gravitate",
      ParticleActionType::Gravity => "Gravity",
      ParticleActionType::Jet => "Jet",
      ParticleActionType::KillOld => "KillOld",
      ParticleActionType::MatchVelocity => "MatchVelocity",
      ParticleActionType::Move => "Move",
      ParticleActionType::OrbitLine => "OrbitLine",
      ParticleActionType::OrbitPoint => "OrbitPoint",
      ParticleActionType::RandomAccel => "RandomAccel",
      ParticleActionType::RandomDisplace => "RandomDisplace",
      ParticleActionType::RandomVelocity => "RandomVelocity",
      ParticleActionType::Restore => "Restore",
      ParticleActionType::Scatter => "Scatter",
      ParticleActionType::Sink => "Sink",
      ParticleActionType::SinkVelocity => "SinkVelocity",
      ParticleActionType::Source => "Source",
      ParticleActionType::SpeedLimit => "SpeedLimit",
      ParticleActionType::TargetColor => "TargetColor",
      ParticleActionType::TargetSize => "TargetSize",
      ParticleActionType::TargetRotate | ParticleActionType::TargetRotateD => "TargetRotate",
      ParticleActionType::TargetVelocity | ParticleActionType::TargetVelocityD => "TargetVelocity",
      ParticleActionType::Vortex => "Vortex",
      ParticleActionType::Turbulence => "Turbulence",
      ParticleActionType::CallActionList | ParticleActionType::Unknown => {
        return Err(XrfError::new_invalid_error(format!(
          "No particle editor action is of type {action_type}"
        )));
      }
    })
  }

  /// A kind of action's properties at their starting values, in the order the SDK shows them.
  ///
  /// # Errors
  ///
  /// Returns an error for a type no editor action is made of, or a version no SDK wrote.
  pub fn get_defaults(action_type: ParticleActionType, version: u32) -> XrfResult<Vec<ParticleEditorProperty>> {
    if version > Self::VERSION {
      return Err(XrfError::new_invalid_error(format!(
        "Particle editor action version {version} is newer than the {} the SDK writes",
        Self::VERSION
      )));
    }

    Self::get_type_name(action_type)?;

    Ok(match action_type {
      ParticleActionType::Avoid => vec![
        Self::domain("Position"),
        Self::float("Magnitude", 0.0),
        Self::float("Epsilon", 0.0),
        Self::float("Look Ahead", 0.0),
        Self::boolean("Allow Rotate", true),
      ],
      ParticleActionType::Bounce => vec![
        Self::domain("Position"),
        Self::float("Friction", 0.5),
        Self::float("Resilience", 0.1),
        Self::float("Cutoff", 1.0),
        Self::boolean("Allow Rotate", true),
      ],
      ParticleActionType::CopyVertex => vec![Self::boolean("Copy Position", true)],
      ParticleActionType::Damping => vec![
        Self::vector("Damping", 0.0, 0.0, 0.0),
        Self::float("V Low", 0.0),
        Self::float("V High", Self::UNBOUNDED),
      ],
      ParticleActionType::Explosion => vec![
        Self::vector("Center", 0.0, 0.0, 0.0),
        Self::float("Velocity", 1.0),
        Self::float("Magnitude", 2.0),
        Self::float("Standart Dev", 3.0),
        Self::float("Epsilon", Self::SOFTENING),
        Self::float("Age", 0.0),
        Self::boolean("Allow Rotate", true),
      ],
      ParticleActionType::Follow | ParticleActionType::MatchVelocity => vec![
        Self::float("Magnitude", 0.0),
        Self::float("Epsilon", Self::SOFTENING),
        Self::float("Max Radius", Self::UNBOUNDED),
      ],
      ParticleActionType::Gravitate => vec![
        Self::float("Magnitude", 1.0),
        Self::float("Epsilon", 0.001),
        Self::float("Max Radius", 10.0),
      ],
      ParticleActionType::Gravity => vec![
        Self::vector("Direction", 0.0, -9.8, 0.0),
        Self::boolean("Allow Rotate", true),
      ],
      ParticleActionType::Jet => vec![
        Self::domain("Accelerate"),
        Self::vector("Center", 0.0, 0.0, 0.0),
        Self::float("Magnitude", 0.0),
        Self::float("Epsilon", Self::SOFTENING),
        Self::float("Max Radius", Self::UNBOUNDED),
        Self::boolean("Allow Rotate", true),
      ],
      ParticleActionType::KillOld => vec![Self::float("Age Limit", 5.0), Self::boolean("Kill Less Than", false)],
      ParticleActionType::Move => Vec::new(),
      ParticleActionType::OrbitLine => vec![
        Self::vector("Position", 0.0, 0.0, 0.0),
        Self::vector("Axis", 0.0, 0.0, 0.0),
        Self::float("Magnitude", 1.0),
        Self::float("Epsilon", Self::SOFTENING),
        Self::float("Max Radius", Self::UNBOUNDED),
        Self::boolean("Allow Rotate", true),
      ],
      ParticleActionType::OrbitPoint => vec![
        Self::vector("Center", 0.0, 0.0, 0.0),
        Self::float("Magnitude", 400.0),
        Self::float("Epsilon", 0.1),
        Self::float("Max Radius", 100.0),
        Self::boolean("Allow Rotate", true),
      ],
      ParticleActionType::RandomAccel => vec![Self::domain("Accelerate"), Self::boolean("Allow Rotate", true)],
      ParticleActionType::RandomDisplace => vec![Self::domain("Displace"), Self::boolean("Allow Rotate", true)],
      ParticleActionType::RandomVelocity => vec![Self::domain("Velocity"), Self::boolean("Allow Rotate", true)],
      ParticleActionType::Restore => vec![Self::float("Time", 0.0)],
      ParticleActionType::Scatter => vec![
        Self::vector("Center", 0.0, 0.0, 0.0),
        Self::float("Magnitude", 0.0),
        Self::float("Epsilon", Self::SOFTENING),
        Self::float("Max Radius", Self::UNBOUNDED),
        Self::boolean("Allow Rotate", true),
      ],
      ParticleActionType::Sink | ParticleActionType::SinkVelocity => vec![
        Self::boolean("Kill Inside", true),
        Self::domain("Domain"),
        Self::boolean("Allow Rotate", true),
      ],
      ParticleActionType::Source => vec![
        Self::float("Rate", 100.0),
        Self::domain("Domain"),
        Self::domain("Velocity"),
        Self::domain("Rotation"),
        Self::domain("Size"),
        Self::boolean("Single Size", false),
        ParticleEditorProperty::new(
          "Color",
          ParticleEditorValue::Domain(ParticleEditorDomain::new_filled(0, 1.0)),
        ),
        Self::float("Color\\Alpha", 0.0),
        Self::float("Starting Age", 0.0),
        Self::float("Age Sigma", 0.0),
        Self::float("Parent Motion", 0.0),
        Self::boolean("Allow Rotate", false),
      ],
      ParticleActionType::SpeedLimit => vec![Self::float("Min Speed", -1.0), Self::float("Max Speed", 15.0)],
      ParticleActionType::TargetColor if version == Self::SHADOW_OF_CHERNOBYL_VERSION => vec![
        Self::vector("Color", 1.0, 1.0, 1.0),
        Self::float("Alpha", 1.0),
        Self::float("Scale", 1.0),
      ],
      ParticleActionType::TargetColor => vec![
        Self::vector("Color", 1.0, 1.0, 1.0),
        Self::float("Alpha", 1.0),
        Self::float("Scale", 1.0),
        Self::float("TimeFrom", 0.0),
        Self::float("TimeTo", 1.0),
      ],
      ParticleActionType::TargetSize => vec![
        Self::vector("Size", 2.0, 2.0, 0.001),
        Self::vector("Scale", 1.0, 1.0, 0.0),
      ],
      ParticleActionType::TargetRotate | ParticleActionType::TargetRotateD => {
        vec![Self::vector("Rotation", 0.0, 0.0, 0.0), Self::float("Scale", 1.0)]
      }
      ParticleActionType::TargetVelocity | ParticleActionType::TargetVelocityD => vec![
        Self::vector("Velocity", 0.0, 0.0, 0.0),
        Self::float("Scale", 1.0),
        Self::boolean("Allow Rotate", true),
      ],
      ParticleActionType::Vortex => vec![
        Self::vector("Center", 0.0, 0.0, 0.0),
        Self::vector("Axis", 0.0, 1.0, 0.0),
        Self::float("Magnitude", 1.0),
        Self::float("Epsilon", Self::SOFTENING),
        Self::float("Max Radius", 1.0),
        Self::boolean("Allow Rotate", true),
      ],
      ParticleActionType::Turbulence => vec![
        Self::float("Frequency", 2.0),
        ParticleEditorProperty::new("Octaves", ParticleEditorValue::Int(1)),
        Self::float("Magnitude", 10.0),
        Self::float("Delta", 0.01),
        Self::vector("Movement", 1.0, 1.0, 1.0),
      ],
      ParticleActionType::CallActionList | ParticleActionType::Unknown => Vec::new(),
    })
  }

  fn float(name: &str, value: f32) -> ParticleEditorProperty {
    ParticleEditorProperty::new(name, ParticleEditorValue::Float(value))
  }

  fn vector(name: &str, x: f32, y: f32, z: f32) -> ParticleEditorProperty {
    ParticleEditorProperty::new(name, ParticleEditorValue::Vector(Vector3d::new(x, y, z)))
  }

  fn domain(name: &str) -> ParticleEditorProperty {
    ParticleEditorProperty::new(name, ParticleEditorValue::Domain(ParticleEditorDomain::new_point()))
  }

  fn boolean(name: &str, value: bool) -> ParticleEditorProperty {
    ParticleEditorProperty::new(name, ParticleEditorValue::Bool(value))
  }
}
