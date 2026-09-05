/// How a live detail association is applied, from the texture param flags (`TextureDescrManager.cpp:175`).
///
/// The engine keeps the two flags apart in `texture_assoc::usage` and the blender reads them apart too: a diffuse
/// detail multiplies the base colour, a bump detail also brings its own bump pair (`uber_deffer.cpp:43`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThmDetailUsage {
  Diffuse,
  Bump,
  DiffuseAndBump,
}
