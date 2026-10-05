/// One member of a shared struct as WGSL lays it out.
#[derive(Clone, Copy, Debug)]
pub struct ShaderMember {
  pub name: &'static str,
  pub offset: u64,
  pub size: u64,
  pub get_wgsl_name: fn() -> String,
}
