use std::collections::HashSet;

use crate::xray_lua_binding::XRayLuaBinding;

/// The locals and parameters in scope at one point of a walk over a script, the innermost last.
#[derive(Debug, Default)]
pub(crate) struct LuaScope {
  /// One frame a block, a function body or a loop, each holding what it declares in declaration order: the name,
  /// where its declaration is written, and what it binds.
  frames: Vec<Vec<(String, usize, XRayLuaBinding)>>,
  /// Declarations an assignment gives another value somewhere in the script, by where each is written: what such a
  /// name holds where it is read depends on the order the script runs in, which a walk in source order cannot know.
  reassigned: HashSet<usize>,
}

impl LuaScope {
  /// A scope treating the declarations written at `reassigned` as holding nothing it can know.
  pub(crate) fn of_reassigned(reassigned: HashSet<usize>) -> Self {
    Self {
      frames: Vec::new(),
      reassigned,
    }
  }

  /// Enters a block, a function body or a loop.
  pub(crate) fn push(&mut self) {
    self.frames.push(Vec::new());
  }

  /// Leaves the innermost of them, and everything it declared.
  pub(crate) fn pop(&mut self) {
    self.frames.pop();
  }

  /// Declares a name written at byte `at` in the innermost frame, shadowing any it held before.
  pub(crate) fn bind(&mut self, name: String, at: usize, binding: XRayLuaBinding) {
    let binding: XRayLuaBinding = if self.reassigned.contains(&at) {
      XRayLuaBinding::Other
    } else {
      binding
    };

    if let Some(frame) = self.frames.last_mut() {
      frame.push((name, at, binding));
    }
  }

  /// What a name means here: its innermost, latest declaration, or `None` for a global.
  pub(crate) fn resolve(&self, name: &str) -> Option<&XRayLuaBinding> {
    self.find(name).map(|(_, _, binding)| binding)
  }

  /// Where the declaration a name means here is written, or `None` for a global.
  pub(crate) fn resolve_declaration(&self, name: &str) -> Option<usize> {
    self.find(name).map(|(_, at, _)| *at)
  }

  fn find(&self, name: &str) -> Option<&(String, usize, XRayLuaBinding)> {
    self
      .frames
      .iter()
      .rev()
      .flat_map(|frame| frame.iter().rev())
      .find(|(declared, _, _)| declared == name)
  }
}
