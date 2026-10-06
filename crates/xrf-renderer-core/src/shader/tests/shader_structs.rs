use glam::{Mat4, UVec4, Vec3, Vec4};

use crate::shader::{ShaderAddressSpace, ShaderDeclarations, ShaderLayoutVerifier, ShaderStruct, ShaderType};

/// A `vec3` followed by a scalar packs into sixteen bytes in WGSL and in Rust alike.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, crate::ShaderStruct)]
struct Light {
  position: Vec3,
  radius: f32,
  color: Vec4,
  flags: UVec4,
}

/// A struct holding another and an array of it, its end padded to WGSL's size.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, crate::ShaderStruct)]
struct Frame {
  view: Mat4,
  lights: [Light; 2],
  count: u32,
  _end: [u32; 3],
}

/// An array of scalars: fine in a storage buffer, refused in a uniform one, whose arrays step sixteen bytes.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, crate::ShaderStruct)]
struct Counts {
  values: [u32; 4],
}

#[test]
fn lays_a_struct_out_as_wgsl_does() {
  let offsets: Vec<(&str, u64)> = Light::MEMBERS
    .iter()
    .map(|member| (member.name, member.offset))
    .collect();

  assert_eq!(offsets, [("position", 0), ("radius", 12), ("color", 16), ("flags", 32)]);
  assert_eq!((Light::ALIGN, Light::SIZE), (16, 48));
  assert_eq!((Frame::ALIGN, Frame::SIZE), (16, 176));
  assert_eq!(Frame::MEMBERS[2].offset, 160);
}

#[test]
fn leaves_padding_out_of_the_declaration() {
  assert_eq!(
    Frame::get_wgsl_declaration(),
    "struct Frame {\n  view: mat4x4<f32>,\n  lights: array<Light, 2>,\n  count: u32,\n}\n"
  );
}

#[test]
fn declares_a_held_struct_first_and_once() {
  let wgsl: String = ShaderDeclarations::new()
    .declare::<Frame>()
    .declare::<Light>()
    .to_wgsl();

  assert_eq!(wgsl.matches("struct Light").count(), 1);
  assert!(wgsl.find("struct Light").unwrap() < wgsl.find("struct Frame").unwrap());
}

#[test]
fn agrees_with_naga_in_both_address_spaces() {
  ShaderLayoutVerifier::verify::<Light>(ShaderAddressSpace::Uniform).unwrap();
  ShaderLayoutVerifier::verify::<Frame>(ShaderAddressSpace::Uniform).unwrap();
  ShaderLayoutVerifier::verify::<Frame>(ShaderAddressSpace::Storage).unwrap();
  ShaderLayoutVerifier::verify::<Counts>(ShaderAddressSpace::Storage).unwrap();
}

#[test]
fn refuses_what_a_uniform_buffer_cannot_hold() {
  let error: String = ShaderLayoutVerifier::verify::<Counts>(ShaderAddressSpace::Uniform)
    .unwrap_err()
    .to_string();

  assert!(error.contains("naga refuses `Counts` bound as Uniform"), "{error}");
}

#[test]
fn rewrites_a_stale_generated_file_once() {
  let file: crate::GeneratedShaderFile = crate::GeneratedShaderFile::new(
    xrf_test_utils::utils::build_absolute_generated_test_resource_path("generated_shader_file/frame.wgsl"),
  );
  let wgsl: String = ShaderDeclarations::new().declare::<Frame>().to_wgsl();

  assert!(file.sync(&wgsl).unwrap(), "a missing file is written");
  assert!(!file.sync(&wgsl).unwrap(), "an up to date file is left");
  assert!(
    file
      .sync(
        "struct Other { value: f32, }
"
      )
      .unwrap(),
    "a stale file is rewritten"
  );
}
