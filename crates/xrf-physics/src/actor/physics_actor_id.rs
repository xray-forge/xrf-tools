use oxijolt::CharacterId;

/// An actor of a [`crate::PhysicsWorld`], valid until it is removed and only in the world that made it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PhysicsActorId(pub(crate) CharacterId);
