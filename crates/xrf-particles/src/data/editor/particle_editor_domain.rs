use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_math::Vector3d;

use crate::data::particle_domain::ParticleDomain;
use crate::simulation::domain::engine_vector::EngineVector;
use crate::simulation::domain::particle_volume_kind::ParticleVolumeKind;

/// A region as the SDK edits it (`PDomain`): its shape and the nine numbers the shape is given by, three to a point.
///
/// The engine never reads this form; [`Self::compile`] turns it into the [`ParticleDomain`] an action stores.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticleEditorDomain {
  /// `PDomainEnum`: point 0, line 1, triangle 2, plane 3, box 4, sphere 5, cylinder 6, cone 7, blob 8, disc 9,
  /// rectangle 10.
  pub domain_type: u32,
  /// `v[0]` to `v[2]`, which each shape reads its own way: corners, a centre and a normal, or radii in `x` and `y`.
  pub points: [Vector3d; 3],
}

impl ParticleEditorDomain {
  /// `1 / sqrt(2 pi)`, which a blob's falloff is scaled by.
  const ONE_OVER_SQRT_2PI: f32 = 1.0 / 2.506_628_3;

  /// A point at the origin, which every domain property starts as.
  pub fn new_point() -> Self {
    Self::new_filled(0, 0.0)
  }

  /// A domain whose nine numbers are all `value`, as the source's colour starts at white.
  pub fn new_filled(domain_type: u32, value: f32) -> Self {
    let point: Vector3d = Vector3d::new(value, value, value);

    Self {
      domain_type,
      points: [point.clone(), point.clone(), point],
    }
  }

  /// The domain an action stores, as `pDomain`'s constructor derives it (`xrParticles/particle_core.cpp`).
  ///
  /// Fields a shape does not derive are zero; the SDK leaves them as whatever memory held, which no shape reads.
  pub fn compile(&self) -> ParticleDomain {
    let [a, b, c] = &self.points;
    let mut domain: ParticleDomain = ParticleDomain {
      domain_type: self.domain_type,
      coordinates: (Vector3d::ZERO, Vector3d::ZERO),
      basis: (Vector3d::ZERO, Vector3d::ZERO),
      radius1: 0.0,
      radius2: 0.0,
      radius1_sqr: 0.0,
      radius2_sqr: 0.0,
    };

    match ParticleVolumeKind::from(self.domain_type) {
      ParticleVolumeKind::Point => domain.coordinates.0 = a.clone(),
      ParticleVolumeKind::Line => domain.coordinates = (a.clone(), b - a),
      ParticleVolumeKind::Box => {
        let (low_x, high_x) = Self::order_pair(a.x, b.x);
        let (low_y, high_y) = Self::order_pair(a.y, b.y);
        let (low_z, high_z) = Self::order_pair(a.z, b.z);

        domain.coordinates = (
          Vector3d::new(low_x, low_y, low_z),
          Vector3d::new(high_x, high_y, high_z),
        );
      }
      ParticleVolumeKind::Triangle => Self::compile_planar(&mut domain, a.clone(), b - a, c - a),
      ParticleVolumeKind::Rectangle => Self::compile_planar(&mut domain, a.clone(), b.clone(), c.clone()),
      ParticleVolumeKind::Plane => {
        let normal: Vector3d = b.clone().normalize_safe();

        domain.radius1 = -a.dot(&normal);
        domain.coordinates = (a.clone(), normal);
      }
      ParticleVolumeKind::Sphere => {
        (domain.radius1, domain.radius2) = Self::order_radii(b.x, b.y);
        domain.radius1_sqr = domain.radius1 * domain.radius1;
        domain.radius2_sqr = domain.radius2 * domain.radius2;
        domain.coordinates.0 = a.clone();
      }
      ParticleVolumeKind::Cylinder | ParticleVolumeKind::Cone => {
        let axis: Vector3d = b - a;
        let axis_length_squared: f32 = axis.length_squared();
        let normal: Vector3d = axis.clone().normalize_safe();
        let (u, v) = Self::orthonormal_basis(&normal);

        (domain.radius1, domain.radius2) = Self::order_radii(c.x, c.y);
        domain.radius1_sqr = domain.radius1 * domain.radius1;
        // Holds `1 / (p2 . p2)` rather than a radius, which the cone and cylinder tests divide by.
        domain.radius2_sqr = if axis_length_squared != 0.0 {
          1.0 / axis_length_squared
        } else {
          0.0
        };
        domain.coordinates = (a.clone(), axis);
        domain.basis = (u, v);
      }
      ParticleVolumeKind::Blob => {
        let inverse: f32 = 1.0 / b.x;

        domain.radius1 = b.x;
        domain.radius2_sqr = -0.5 * inverse * inverse;
        domain.radius2 = Self::ONE_OVER_SQRT_2PI * inverse;
        domain.coordinates.0 = a.clone();
      }
      ParticleVolumeKind::Disc => {
        let normal: Vector3d = b.clone().normalize_safe();
        let (u, v) = Self::orthonormal_basis(&normal);

        (domain.radius1, domain.radius2) = Self::order_radii(c.x, c.y);
        // Holds the plane equation's `d` rather than a radius.
        domain.radius1_sqr = -a.dot(&normal);
        domain.coordinates = (a.clone(), normal);
        domain.basis = (u, v);
      }
      ParticleVolumeKind::Unknown(_) => {}
    }

    domain
  }

  /// A triangle or rectangle: a corner, two edges, the unit normal, and the plane equation's `d`.
  fn compile_planar(domain: &mut ParticleDomain, corner: Vector3d, u: Vector3d, v: Vector3d) {
    // The edges' lengths are kept in the squared radii's places, which the bounce tests read as such.
    domain.radius1_sqr = u.length();
    domain.radius2_sqr = v.length();

    let normal: Vector3d = (&u / domain.radius1_sqr)
      .cross(&(&v / domain.radius2_sqr))
      .normalize_safe();

    domain.radius1 = -corner.dot(&normal);
    domain.coordinates = (corner, normal);
    domain.basis = (u, v);
  }

  /// Two unit vectors across `normal`, built from the X axis unless it lies almost along it.
  fn orthonormal_basis(normal: &Vector3d) -> (Vector3d, Vector3d) {
    let mut basis: Vector3d = Vector3d::X;

    if basis.dot(normal).abs() > 0.999 {
      basis = Vector3d::Y;
    }

    let u: Vector3d = (&basis - &(normal * basis.dot(normal))).normalize_safe();
    let v: Vector3d = normal.cross(&u);

    (u, v)
  }

  /// The pair low first, as the constructor orders a box's corners: equal values keep the second first.
  fn order_pair(first: f32, second: f32) -> (f32, f32) {
    if first < second {
      (first, second)
    } else {
      (second, first)
    }
  }

  /// The pair high first, as the constructor orders an outer and inner radius: equal values keep the second first.
  fn order_radii(first: f32, second: f32) -> (f32, f32) {
    if first > second {
      (first, second)
    } else {
      (second, first)
    }
  }
}

impl ChunkReadWrite for ParticleEditorDomain {
  /// Reads a domain as `PDomain::Load` does: its shape and its three points.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      domain_type: reader.read_u32::<T>()?,
      points: [
        reader.read_xr::<T, _>()?,
        reader.read_xr::<T, _>()?,
        reader.read_xr::<T, _>()?,
      ],
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u32::<T>(self.domain_type)?;

    for point in &self.points {
      writer.write_xr::<T, _>(point)?;
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use xrf_math::Vector3d;

  use crate::data::editor::particle_editor_domain::ParticleEditorDomain;
  use crate::data::particle_domain::ParticleDomain;

  fn domain(domain_type: u32, points: [[f32; 3]; 3]) -> ParticleEditorDomain {
    ParticleEditorDomain {
      domain_type,
      points: points.map(Vector3d::from),
    }
  }

  #[test]
  fn compiles_a_line_as_its_start_and_the_way_to_its_end() {
    let compiled: ParticleDomain = domain(1, [[1.0, 2.0, 3.0], [4.0, 6.0, 3.0], [0.0; 3]]).compile();

    assert_eq!(
      compiled.coordinates,
      (Vector3d::new(1.0, 2.0, 3.0), Vector3d::new(3.0, 4.0, 0.0))
    );
  }

  #[test]
  fn compiles_a_box_with_its_corners_ordered_per_axis() {
    let compiled: ParticleDomain = domain(4, [[5.0, -1.0, 2.0], [1.0, 3.0, 2.0], [0.0; 3]]).compile();

    assert_eq!(
      compiled.coordinates,
      (Vector3d::new(1.0, -1.0, 2.0), Vector3d::new(5.0, 3.0, 2.0))
    );
  }

  #[test]
  fn compiles_a_sphere_with_the_larger_radius_outside() {
    let compiled: ParticleDomain = domain(5, [[0.0; 3], [1.0, 3.0, 0.0], [0.0; 3]]).compile();

    assert_eq!((compiled.radius1, compiled.radius2), (3.0, 1.0));
    assert_eq!((compiled.radius1_sqr, compiled.radius2_sqr), (9.0, 1.0));
  }

  #[test]
  fn compiles_a_plane_with_its_unit_normal_and_offset() {
    let compiled: ParticleDomain = domain(3, [[0.0, 2.0, 0.0], [0.0, 5.0, 0.0], [0.0; 3]]).compile();

    assert_eq!(compiled.coordinates.1, Vector3d::Y);
    assert_eq!(compiled.radius1, -2.0);
  }

  #[test]
  fn compiles_a_cylinder_with_a_basis_across_its_axis() {
    // An axis along X turns the basis from Y, since crossing X with itself gives nothing.
    let compiled: ParticleDomain = domain(6, [[0.0; 3], [2.0, 0.0, 0.0], [1.0, 0.5, 0.0]]).compile();

    assert_eq!(compiled.coordinates.1, Vector3d::new(2.0, 0.0, 0.0));
    assert_eq!(compiled.radius2_sqr, 0.25);
    assert_eq!(compiled.basis.0.dot(&Vector3d::X), 0.0);
    assert_eq!(compiled.basis.0.length(), 1.0);
  }

  #[test]
  fn compiles_a_triangle_with_its_normal_and_edge_lengths() {
    let compiled: ParticleDomain = domain(2, [[0.0; 3], [2.0, 0.0, 0.0], [0.0, 0.0, 4.0]]).compile();

    assert_eq!(compiled.coordinates.1, Vector3d::new(0.0, -1.0, 0.0));
    assert_eq!((compiled.radius1_sqr, compiled.radius2_sqr), (2.0, 4.0));
  }
}
