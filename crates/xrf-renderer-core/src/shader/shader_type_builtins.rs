//! `ShaderType` for the types WGSL lays out as Rust does: scalars, glam's vectors and matrices, and arrays.
//!
//! A Rust array is a WGSL `array`, never a vector: `[u32; 4]` is `array<u32, 4>`, `UVec4` is `vec4<u32>`. glam's `Mat3`
//! is three packed columns where WGSL pads each to sixteen bytes, so only `Mat3A` maps to `mat3x3<f32>`; `Vec3A`,
//! sixteen bytes where WGSL's `vec3` is twelve, maps to nothing.

use crate::shader::shader_declarations::ShaderDeclarations;
use crate::shader::shader_layout::round_up;
use crate::shader::shader_type::ShaderType;

macro_rules! shader_type {
  ($rust:ty, $wgsl:literal, $align:literal, $size:literal) => {
    impl ShaderType for $rust {
      const ALIGN: u64 = $align;
      const SIZE: u64 = $size;

      fn get_wgsl_name() -> String {
        String::from($wgsl)
      }
    }
  };
}

shader_type!(f32, "f32", 4, 4);
shader_type!(u32, "u32", 4, 4);
shader_type!(i32, "i32", 4, 4);
shader_type!(glam::Vec2, "vec2<f32>", 8, 8);
shader_type!(glam::Vec3, "vec3<f32>", 16, 12);
shader_type!(glam::Vec4, "vec4<f32>", 16, 16);
shader_type!(glam::UVec2, "vec2<u32>", 8, 8);
shader_type!(glam::UVec3, "vec3<u32>", 16, 12);
shader_type!(glam::UVec4, "vec4<u32>", 16, 16);
shader_type!(glam::IVec2, "vec2<i32>", 8, 8);
shader_type!(glam::IVec3, "vec3<i32>", 16, 12);
shader_type!(glam::IVec4, "vec4<i32>", 16, 16);
shader_type!(glam::Mat3A, "mat3x3<f32>", 16, 48);
shader_type!(glam::Mat4, "mat4x4<f32>", 16, 64);

impl<T: ShaderType, const N: usize> ShaderType for [T; N] {
  const ALIGN: u64 = T::ALIGN;
  const SIZE: u64 = round_up(T::ALIGN, T::SIZE) * N as u64;

  fn get_wgsl_name() -> String {
    format!("array<{}, {N}>", T::get_wgsl_name())
  }

  fn declare(declarations: &mut ShaderDeclarations) {
    T::declare(declarations);
  }
}
