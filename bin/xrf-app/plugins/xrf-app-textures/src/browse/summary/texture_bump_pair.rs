use serde::Serialize;
use xrf_material::XrayBumpNaming;

/// The two references a declaration binds, which is what folds a pair under the texture that declares it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureBumpPair {
  pub bump: String,
  pub companion: String,
}

impl TextureBumpPair {
  /// The pair the SDK's generator writes for `texture`: its bump, and the companion the engine derives from it.
  pub fn generated_for(texture: &str) -> Self {
    let bump: String = XrayBumpNaming::bump_of(texture);

    Self {
      companion: XrayBumpNaming::companion_of(&bump),
      bump,
    }
  }
}
