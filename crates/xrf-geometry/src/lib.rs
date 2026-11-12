#![doc = include_str!("../README.md")]

pub(crate) mod ray;
pub(crate) mod tracer;
pub(crate) mod triangle;

pub use crate::ray::facing::Facing;
pub use crate::ray::ray::Ray;
pub use crate::ray::ray_crossing::RayCrossing;
pub use crate::tracer::bvh_tracer::BvhTracer;
pub use crate::tracer::trace_hit::TraceHit;
pub use crate::tracer::tracer::Tracer;
pub use crate::triangle::triangle::Triangle;
