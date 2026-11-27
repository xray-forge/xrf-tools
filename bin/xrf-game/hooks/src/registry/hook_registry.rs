use std::collections::BTreeMap;
use std::fmt::{self, Debug, Formatter};
use std::sync::Arc;

use xrf_error::{XrfError, XrfResult};

use crate::registry::hook_entry::HookEntry;
use crate::registry::hook_kind::HookKind;
use crate::registry::hook_point::HookPoint;

/// Every hook point of the game, by name: the one registry through which a system calls its own behaviour where a mod
/// may change it (plan decision 26). A system registers each of its points with its Rust default as it is built, and
/// reads the entry back each time it acts there, never calling the default past it; a script replaces an entry by
/// name, holding the one it replaced to wrap it, through the same calls. A point's function type is checked on every
/// read and replacement, so an override of the wrong shape is refused, never called.
///
/// Events are told after the fact, in id order, from a system's `EventBuffer`; guards may refuse an action; modifiers
/// may change a value; condlist conditions and effects are entries named after X-Ray's script tables
/// (`xr_conditions.<name>`, `xr_effects.<name>`).
#[derive(Clone, Default)]
pub struct HookRegistry {
  entries: BTreeMap<String, HookEntry>,
}

impl HookRegistry {
  /// Declares `point` with `function` as its default.
  ///
  /// # Errors
  ///
  /// Returns an invalid error where the point is declared already.
  pub fn register<H>(&mut self, point: &HookPoint<H>, function: Arc<H>) -> XrfResult<()>
  where
    H: ?Sized + Send + Sync + 'static,
  {
    self.register_named(point.name, point.kind, function)
  }

  /// Declares the point `name` of `kind`, a name made as the game runs (a condlist function's), with `function` as
  /// its default.
  ///
  /// # Errors
  ///
  /// Returns an invalid error where the point is declared already.
  pub fn register_named<H>(&mut self, name: &str, kind: HookKind, function: Arc<H>) -> XrfResult<()>
  where
    H: ?Sized + Send + Sync + 'static,
  {
    if self.entries.contains_key(name) {
      return Err(XrfError::new_invalid_error(format!(
        "Hook '{name}' is registered already"
      )));
    }

    self.entries.insert(
      String::from(name),
      HookEntry {
        kind,
        function: Arc::new(function),
      },
    );

    Ok(())
  }

  /// Replaces the entry of the point `name` with `function`, answering the function it held, so the new one may call
  /// it as it wraps it.
  ///
  /// # Errors
  ///
  /// Returns a not found error where no point is named so, and an invalid error where its function is of another type.
  pub fn replace<H>(&mut self, name: &str, function: Arc<H>) -> XrfResult<Arc<H>>
  where
    H: ?Sized + Send + Sync + 'static,
  {
    let entry: &mut HookEntry = self
      .entries
      .get_mut(name)
      .ok_or_else(|| XrfError::new_not_found_error(format!("No hook '{name}' to replace")))?;
    let previous: Arc<H> = entry
      .function
      .downcast_ref::<Arc<H>>()
      .cloned()
      .ok_or_else(|| XrfError::new_invalid_error(format!("Hook '{name}' takes a function of another type")))?;

    entry.function = Arc::new(function);

    Ok(previous)
  }

  /// The function `point`'s entry holds now.
  ///
  /// # Errors
  ///
  /// Returns a not found error where the point was never registered, a fault of the game's set-up rather than of a
  /// script, as every point is registered with its default before the game steps.
  pub fn get<H>(&self, point: &HookPoint<H>) -> XrfResult<Arc<H>>
  where
    H: ?Sized + Send + Sync + 'static,
  {
    self
      .find(point.name)
      .ok_or_else(|| XrfError::new_not_found_error(format!("Hook '{}' is not registered", point.name)))
  }

  /// The function the point `name` holds now, none where no point is named so or its function is of another type.
  pub fn find<H>(&self, name: &str) -> Option<Arc<H>>
  where
    H: ?Sized + Send + Sync + 'static,
  {
    self.entries.get(name)?.function.downcast_ref::<Arc<H>>().cloned()
  }

  /// What the point `name` is for, none where no point is named so.
  pub fn get_kind(&self, name: &str) -> Option<HookKind> {
    self.entries.get(name).map(|entry| entry.kind)
  }

  /// How many points it holds.
  pub fn len(&self) -> usize {
    self.entries.len()
  }

  pub fn is_empty(&self) -> bool {
    self.entries.is_empty()
  }
}

impl Debug for HookRegistry {
  /// The points by name with their kinds; functions show nothing of themselves.
  fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
    formatter
      .debug_map()
      .entries(self.entries.iter().map(|(name, entry)| (name, entry.kind)))
      .finish()
  }
}
