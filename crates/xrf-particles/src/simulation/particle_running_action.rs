use glam::{Mat4, Vec3};

use crate::data::particle_action::ParticleAction;
use crate::simulation::actions::avoid_action::AvoidAction;
use crate::simulation::actions::bounce_action::BounceAction;
use crate::simulation::actions::copy_vertex_action::CopyVertexAction;
use crate::simulation::actions::damping_action::DampingAction;
use crate::simulation::actions::explosion_action::ExplosionAction;
use crate::simulation::actions::follow_action::FollowAction;
use crate::simulation::actions::gravitate_action::GravitateAction;
use crate::simulation::actions::gravity_action::GravityAction;
use crate::simulation::actions::jet_action::JetAction;
use crate::simulation::actions::kill_old_action::KillOldAction;
use crate::simulation::actions::match_velocity_action::MatchVelocityAction;
use crate::simulation::actions::move_action::MoveAction;
use crate::simulation::actions::orbit_line_action::OrbitLineAction;
use crate::simulation::actions::orbit_point_action::OrbitPointAction;
use crate::simulation::actions::particle_action_step::ParticleActionStep;
use crate::simulation::actions::random_acceleration_action::RandomAccelerationAction;
use crate::simulation::actions::random_displace_action::RandomDisplaceAction;
use crate::simulation::actions::random_velocity_action::RandomVelocityAction;
use crate::simulation::actions::restore_action::RestoreAction;
use crate::simulation::actions::scatter_action::ScatterAction;
use crate::simulation::actions::sink_action::SinkAction;
use crate::simulation::actions::sink_velocity_action::SinkVelocityAction;
use crate::simulation::actions::source_action::SourceAction;
use crate::simulation::actions::speed_limit_action::SpeedLimitAction;
use crate::simulation::actions::target_color_action::TargetColorAction;
use crate::simulation::actions::target_rotate_action::TargetRotateAction;
use crate::simulation::actions::target_size_action::TargetSizeAction;
use crate::simulation::actions::target_velocity_action::TargetVelocityAction;
use crate::simulation::actions::turbulence_action::TurbulenceAction;
use crate::simulation::actions::vortex_action::VortexAction;
use crate::simulation::particle_pool::ParticlePool;

/// One action of a running effect: its own copy, local and placed values, and its state between steps.
pub(crate) struct ParticleRunningAction {
  /// `ALLOW_ROTATE`: placed by the whole matrix rather than by its translation alone.
  is_rotated: bool,
  kind: RunningKind,
}

enum RunningKind {
  Avoid(AvoidAction),
  Bounce(BounceAction),
  CopyVertex(CopyVertexAction),
  Damping(DampingAction),
  Explosion(ExplosionAction),
  Follow(FollowAction),
  Gravitate(GravitateAction),
  Gravity(GravityAction),
  Jet(JetAction),
  KillOld(KillOldAction),
  MatchVelocity(MatchVelocityAction),
  Move(MoveAction),
  OrbitLine(OrbitLineAction),
  OrbitPoint(OrbitPointAction),
  RandomAcceleration(RandomAccelerationAction),
  RandomDisplace(RandomDisplaceAction),
  RandomVelocity(RandomVelocityAction),
  Restore(RestoreAction),
  Scatter(ScatterAction),
  Sink(SinkAction),
  SinkVelocity(SinkVelocityAction),
  Source(Box<SourceAction>),
  SpeedLimit(SpeedLimitAction),
  TargetColor(TargetColorAction),
  TargetRotate(TargetRotateAction),
  TargetSize(TargetSizeAction),
  TargetVelocity(TargetVelocityAction),
  Turbulence(TurbulenceAction),
  Vortex(VortexAction),
}

impl ParticleRunningAction {
  /// `ParticleAction::ALLOW_ROTATE`.
  const ALLOW_ROTATE: u32 = 1 << 1;

  /// `PlayEffect`: sources emit again; explosions and turbulence start over.
  pub fn play(&mut self) {
    match &mut self.kind {
      RunningKind::Source(action) => action.set_silent(false),
      RunningKind::Explosion(action) => action.play(),
      RunningKind::Turbulence(action) => action.play(),
      _ => {}
    }
  }

  /// `StopEffect`: sources fall silent.
  pub fn stop(&mut self) {
    if let RunningKind::Source(action) = &mut self.kind {
      action.set_silent(true);
    }
  }

  /// `ParticleManager::Transform`: places the action, by translation alone unless it allows rotation.
  pub fn transform(&mut self, matrix: &Mat4, velocity: Vec3) {
    let placement: Mat4 = if self.is_rotated {
      *matrix
    } else {
      Mat4::from_translation(matrix.w_axis.truncate())
    };

    match &mut self.kind {
      RunningKind::Avoid(action) => action.transform(&placement),
      RunningKind::Bounce(action) => action.transform(&placement),
      RunningKind::Explosion(action) => action.transform(&placement),
      RunningKind::Jet(action) => action.transform(&placement),
      RunningKind::OrbitLine(action) => action.transform(&placement),
      RunningKind::OrbitPoint(action) => action.transform(&placement),
      RunningKind::RandomAcceleration(action) => action.transform(&placement),
      RunningKind::RandomDisplace(action) => action.transform(&placement),
      RunningKind::RandomVelocity(action) => action.transform(&placement),
      RunningKind::Scatter(action) => action.transform(&placement),
      RunningKind::Sink(action) => action.transform(&placement),
      RunningKind::SinkVelocity(action) => action.transform(&placement),
      RunningKind::Source(action) => action.transform(&placement, velocity),
      RunningKind::TargetVelocity(action) => action.transform(&placement),
      RunningKind::Vortex(action) => action.transform(&placement),
      RunningKind::CopyVertex(_)
      | RunningKind::Damping(_)
      | RunningKind::Follow(_)
      | RunningKind::Gravitate(_)
      | RunningKind::Gravity(_)
      | RunningKind::KillOld(_)
      | RunningKind::MatchVelocity(_)
      | RunningKind::Move(_)
      | RunningKind::Restore(_)
      | RunningKind::SpeedLimit(_)
      | RunningKind::TargetColor(_)
      | RunningKind::TargetRotate(_)
      | RunningKind::TargetSize(_)
      | RunningKind::Turbulence(_) => {}
    }
  }

  /// `Execute`.
  pub fn execute(&mut self, pool: &mut ParticlePool, step: &mut ParticleActionStep) {
    match &mut self.kind {
      RunningKind::Avoid(action) => action.execute(pool, step),
      RunningKind::Bounce(action) => action.execute(pool, step),
      RunningKind::CopyVertex(action) => action.execute(pool),
      RunningKind::Damping(action) => action.execute(pool, step),
      RunningKind::Explosion(action) => action.execute(pool, step),
      RunningKind::Follow(action) => action.execute(pool, step),
      RunningKind::Gravitate(action) => action.execute(pool, step),
      RunningKind::Gravity(action) => action.execute(pool, step),
      RunningKind::Jet(action) => action.execute(pool, step),
      RunningKind::KillOld(action) => action.execute(pool, step),
      RunningKind::MatchVelocity(action) => action.execute(pool, step),
      RunningKind::Move(action) => action.execute(pool, step),
      RunningKind::OrbitLine(action) => action.execute(pool, step),
      RunningKind::OrbitPoint(action) => action.execute(pool, step),
      RunningKind::RandomAcceleration(action) => action.execute(pool, step),
      RunningKind::RandomDisplace(action) => action.execute(pool, step),
      RunningKind::RandomVelocity(action) => action.execute(pool),
      RunningKind::Restore(action) => action.execute(pool, step),
      RunningKind::Scatter(action) => action.execute(pool, step),
      RunningKind::Sink(action) => action.execute(pool),
      RunningKind::SinkVelocity(action) => action.execute(pool),
      RunningKind::Source(action) => action.execute(pool, step),
      RunningKind::SpeedLimit(action) => action.execute(pool),
      RunningKind::TargetColor(action) => action.execute(pool, step),
      RunningKind::TargetRotate(action) => action.execute(pool, step),
      RunningKind::TargetSize(action) => action.execute(pool, step),
      RunningKind::TargetVelocity(action) => action.execute(pool, step),
      RunningKind::Turbulence(action) => action.execute(pool, step),
      RunningKind::Vortex(action) => action.execute(pool, step),
    }
  }
}

impl From<&ParticleAction> for ParticleRunningAction {
  fn from(action: &ParticleAction) -> Self {
    let (flags, kind): (u32, RunningKind) = match action {
      ParticleAction::Avoid(it) => (it.action_flags, RunningKind::Avoid(it.as_ref().into())),
      ParticleAction::Bounce(it) => (it.action_flags, RunningKind::Bounce(it.as_ref().into())),
      ParticleAction::CopyVertex(it) => (it.action_flags, RunningKind::CopyVertex(it.as_ref().into())),
      ParticleAction::Damping(it) => (it.action_flags, RunningKind::Damping(it.as_ref().into())),
      ParticleAction::Explosion(it) => (it.action_flags, RunningKind::Explosion(it.as_ref().into())),
      ParticleAction::Follow(it) => (it.action_flags, RunningKind::Follow(it.as_ref().into())),
      ParticleAction::Gravitate(it) => (it.action_flags, RunningKind::Gravitate(it.as_ref().into())),
      ParticleAction::Gravity(it) => (it.action_flags, RunningKind::Gravity(it.as_ref().into())),
      ParticleAction::Jet(it) => (it.action_flags, RunningKind::Jet(it.as_ref().into())),
      ParticleAction::KillOld(it) => (it.action_flags, RunningKind::KillOld(it.as_ref().into())),
      ParticleAction::MatchVelocity(it) => (it.action_flags, RunningKind::MatchVelocity(it.as_ref().into())),
      ParticleAction::Move(it) => (it.action_flags, RunningKind::Move(MoveAction)),
      ParticleAction::OrbitLine(it) => (it.action_flags, RunningKind::OrbitLine(it.as_ref().into())),
      ParticleAction::OrbitPoint(it) => (it.action_flags, RunningKind::OrbitPoint(it.as_ref().into())),
      ParticleAction::RandomAccel(it) => (it.action_flags, RunningKind::RandomAcceleration(it.as_ref().into())),
      ParticleAction::RandomDisplace(it) => (it.action_flags, RunningKind::RandomDisplace(it.as_ref().into())),
      ParticleAction::RandomVelocity(it) => (it.action_flags, RunningKind::RandomVelocity(it.as_ref().into())),
      ParticleAction::Restore(it) => (it.action_flags, RunningKind::Restore(it.as_ref().into())),
      ParticleAction::Scatter(it) => (it.action_flags, RunningKind::Scatter(it.as_ref().into())),
      ParticleAction::Sink(it) => (it.action_flags, RunningKind::Sink(it.as_ref().into())),
      ParticleAction::SinkVelocity(it) => (it.action_flags, RunningKind::SinkVelocity(it.as_ref().into())),
      ParticleAction::Source(it) => (it.action_flags, RunningKind::Source(Box::new(it.as_ref().into()))),
      ParticleAction::SpeedLimit(it) => (it.action_flags, RunningKind::SpeedLimit(it.as_ref().into())),
      ParticleAction::TargetColor(it) => (it.action_flags, RunningKind::TargetColor(it.as_ref().into())),
      ParticleAction::TargetRotate(it) => (it.action_flags, RunningKind::TargetRotate(it.as_ref().into())),
      ParticleAction::TargetSize(it) => (it.action_flags, RunningKind::TargetSize(it.as_ref().into())),
      ParticleAction::TargetVelocity(it) => (it.action_flags, RunningKind::TargetVelocity(it.as_ref().into())),
      ParticleAction::Turbulence(it) => (it.action_flags, RunningKind::Turbulence(it.as_ref().into())),
      ParticleAction::Vortex(it) => (it.action_flags, RunningKind::Vortex(it.as_ref().into())),
    };

    Self {
      is_rotated: flags & Self::ALLOW_ROTATE != 0,
      kind,
    }
  }
}
