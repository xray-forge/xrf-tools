/// What `gbuf_pack_hemi_mtl` divides a material by before keeping 5 bits of it.
const PACKED_RANGE: f32 = 1.333_333_3;

/// The steps a G-buffer's material is kept in over its range.
const PACKED_STEPS: f32 = 31.0;

/// The material table's slice a texture's material is lit by, as the DX10 engines leave it in the G-buffer.
pub fn material_slice(material: f32) -> f32 {
  // `apply_lmaterial`: Anomaly passes a material of five or more as it stands.
  let value: f32 = if material < 5.0 {
    (material + 0.5) / 4.0
  } else {
    material
  };
  let packed: u32 = ((value / PACKED_RANGE) * PACKED_STEPS) as u32 & 31;

  packed as f32 / PACKED_STEPS * PACKED_RANGE
}
