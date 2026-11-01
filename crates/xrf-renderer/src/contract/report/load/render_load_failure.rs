use serde::Serialize;

/// Something of a level that could not be read, by what names it, and why: a sector by its index, a spawned model by
/// its visual's name.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderLoadFailure {
  pub name: String,
  pub reason: String,
}
