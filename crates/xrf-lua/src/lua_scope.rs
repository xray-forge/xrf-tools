use crate::xray_lua_binding::XRayLuaBinding;

/// The locals and parameters in scope at one point of a walk over a script, the innermost last.
#[derive(Debug, Default)]
pub(crate) struct LuaScope {
  /// One frame a block, a function body or a loop, each holding what it declares in declaration order.
  frames: Vec<Vec<(String, XRayLuaBinding)>>,
}

impl LuaScope {
  /// Enters a block, a function body or a loop.
  pub(crate) fn push(&mut self) {
    self.frames.push(Vec::new());
  }

  /// Leaves the innermost of them, and everything it declared.
  pub(crate) fn pop(&mut self) {
    self.frames.pop();
  }

  /// Declares a name in the innermost frame, shadowing any it held before.
  pub(crate) fn bind(&mut self, name: String, binding: XRayLuaBinding) {
    if let Some(frame) = self.frames.last_mut() {
      frame.push((name, binding));
    }
  }

  /// What a name means here: its innermost, latest declaration, or `None` for a global.
  pub(crate) fn resolve(&self, name: &str) -> Option<&XRayLuaBinding> {
    self
      .frames
      .iter()
      .rev()
      .flat_map(|frame| frame.iter().rev())
      .find(|(declared, _)| declared == name)
      .map(|(_, binding)| binding)
  }
}
