use crate::key::declare_environment_keys;

declare_environment_keys! {
  /// The keys of a `thunderbolts.ltx` section, `SThunderboltDesc` (`xrEngine/thunderbolt.cpp`): one bolt a collection
  /// strikes with.
  pub enum ThunderboltKey {
    /// A light animation, `lanims.xr`, that colours the flash over the bolt's life.
    ColorAnim = "color_anim": Text, vanilla: Required(Text("")), extended: Required(Text(""));
    /// A detail model, `.dm`, under `meshes`.
    LightningModel = "lightning_model": Text, vanilla: Required(Text("")), extended: Required(Text(""));
    /// A sound, empty for none.
    Sound = "sound": Text, vanilla: Required(Text("")), extended: Required(Text(""));
    GradientTopOpacity = "gradient_top_opacity": Number,
      vanilla: Required(Number(0.0)), extended: Required(Number(0.0));
    GradientTopRadius = "gradient_top_radius": Vector { least: 2, most: 2 },
      vanilla: Required(Vector(&[0.0, 0.0])), extended: Required(Vector(&[0.0, 0.0]));
    GradientTopShader = "gradient_top_shader": Text, vanilla: Required(Text("")), extended: Required(Text(""));
    GradientTopTexture = "gradient_top_texture": Text, vanilla: Required(Text("")), extended: Required(Text(""));
    GradientCenterOpacity = "gradient_center_opacity": Number,
      vanilla: Required(Number(0.0)), extended: Required(Number(0.0));
    GradientCenterRadius = "gradient_center_radius": Vector { least: 2, most: 2 },
      vanilla: Required(Vector(&[0.0, 0.0])), extended: Required(Vector(&[0.0, 0.0]));
    GradientCenterShader = "gradient_center_shader": Text, vanilla: Required(Text("")), extended: Required(Text(""));
    GradientCenterTexture = "gradient_center_texture": Text,
      vanilla: Required(Text("")), extended: Required(Text(""));
  }
}
