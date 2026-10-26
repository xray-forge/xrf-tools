/// How a live detail association is applied, from the texture param flags (`TextureDescrManager.cpp:175`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThmDetailUsage {
  Diffuse,
  Bump,
  DiffuseAndBump,
}
