use xrf_shaders::ShaderBlender;

/// The three authored values every alpha rule reads out of a blender.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct XraySurfaceAlpha {
  pub(crate) is_used: Option<bool>,
  pub(crate) reference: Option<u8>,
  pub(crate) is_strict_sorting: bool,
}

impl XraySurfaceAlpha {
  /// Reads the knobs a rule names: a switch the file omits as off and a reference it omits as `default`, which is what
  /// each class's constructor starts both from before its `Load` reads them.
  pub(crate) fn read(blender: &ShaderBlender, switch: Option<&str>, reference: Option<(&str, u8)>) -> Self {
    Self {
      is_used: switch.map(|property| blender.boolean(property).unwrap_or(false)),
      reference: reference.map(|(property, default)| blender.integer(property).map_or(default, Self::to_reference)),
      is_strict_sorting: blender.is_strict_sorting(),
    }
  }

  /// Whether the class's switch is set, for a rule that reads it.
  pub(crate) fn is_switched_on(self) -> bool {
    self.is_used.unwrap_or(false)
  }

  /// The authored reference as a render state value.
  fn to_reference(authored: i32) -> u8 {
    authored.clamp(0, i32::from(u8::MAX)) as u8
  }
}
