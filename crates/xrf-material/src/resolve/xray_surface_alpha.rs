use xrf_db::ShaderBlender;

/// The three authored values every alpha rule reads out of a blender.
///
/// The engine's own operands: a class's alpha switch, its alpha reference, and the strict sorting flag `IBlender`
/// writes for every class. Sorting is here because it is only ever read together with the other two - it is what takes
/// a surface out of the deferred path (`blenders/blender_deffer_model.cpp`) - and reading it apart would leave the
/// forward decision split across two places.
///
/// A knob the class does not write is `None` rather than a default, so a rule cannot silently read one that was never
/// authored and a panel can say which knobs the class has.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct XraySurfaceAlpha {
  pub(crate) is_used: Option<bool>,
  pub(crate) reference: Option<u8>,
  pub(crate) is_strict_sorting: bool,
}

impl XraySurfaceAlpha {
  /// Reads the knobs a rule names, taking a switch the file omits as off.
  ///
  /// An omitted switch is what the engine's zero-initialised property would give, which keeps a library written by
  /// something other than the SDK readable rather than describing it as having no switch at all.
  pub(crate) fn read(blender: &ShaderBlender, switch: Option<&str>, reference: Option<&str>) -> Self {
    Self {
      is_used: switch.map(|property| blender.boolean(property).unwrap_or(false)),
      reference: reference
        .and_then(|property| blender.integer(property))
        .map(Self::to_reference),
      is_strict_sorting: blender.is_strict_sorting(),
    }
  }

  /// Whether the class's switch is set, for a rule that reads it.
  pub(crate) fn is_switched_on(self) -> bool {
    self.is_used.unwrap_or(false)
  }

  /// The authored reference as a render state value.
  ///
  /// Clamped rather than refused: the property carries its own `0..255` range and the editor holds authoring to it, so
  /// a value outside it comes from something else and the engine would pass it to a byte wide render state all the
  /// same.
  fn to_reference(authored: i32) -> u8 {
    authored.clamp(0, i32::from(u8::MAX)) as u8
  }
}
