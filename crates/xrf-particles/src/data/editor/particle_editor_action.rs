use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReadWriteList, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_math::Vector3d;
use xrf_utils::to_format_size;

use crate::data::editor::particle_editor_action_compiler::ParticleEditorActionCompiler;
use crate::data::editor::particle_editor_domain::ParticleEditorDomain;
use crate::data::editor::particle_editor_property::ParticleEditorProperty;
use crate::data::editor::particle_editor_schema::ParticleEditorSchema;
use crate::data::editor::particle_editor_value::ParticleEditorValue;
use crate::data::editor::particle_editor_value_kind::ParticleEditorValueKind;
use crate::data::particle_action::ParticleAction;
use crate::data::particle_action_type::ParticleActionType;

/// One action as the SDK's particle editor holds it (`EParticleAction`): named, switchable, and made of named
/// properties rather than the derived values the engine plays.
///
/// An effect keeps its editor actions beside its compiled ones; [`Self::compile`] derives the second from the first, as
/// the SDK does on every save.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticleEditorAction {
  pub action_type: ParticleActionType,
  /// `PARTICLE_ACTION_VERSION` it was saved at, which decides its properties ([`ParticleEditorSchema`]).
  pub version: u32,
  /// The name the editor lists it by, its kind's name unless renamed.
  pub name: String,
  pub flags: u32,
  /// Its properties in the order the SDK shows them, every one its kind has.
  pub properties: Vec<ParticleEditorProperty>,
}

impl ParticleEditorAction {
  /// `flEnabled`: compiled into the effect; a disabled action is kept for editing only.
  pub const FLAG_ENABLED: u32 = 1 << 0;

  /// `flDraw`: its domains are drawn in the editor's viewport.
  pub const FLAG_DRAW: u32 = 1 << 1;

  /// The type, the version, an empty name and the flags, before any property.
  pub const MIN_SERIALIZED_SIZE: u64 = 4 + 4 + 1 + 4;

  /// A new action of a kind at its starting values, enabled, as the SDK adds one.
  ///
  /// # Errors
  ///
  /// Returns an error for a type no editor action is made of.
  pub fn new(action_type: ParticleActionType) -> XrfResult<Self> {
    Ok(Self {
      action_type,
      version: ParticleEditorSchema::VERSION,
      name: String::from(ParticleEditorSchema::get_type_name(action_type)?),
      flags: Self::FLAG_ENABLED,
      properties: ParticleEditorSchema::get_defaults(action_type, ParticleEditorSchema::VERSION)?,
    })
  }

  pub fn is_enabled(&self) -> bool {
    self.flags & Self::FLAG_ENABLED != 0
  }

  /// The value of the property by `name`, where the action has one.
  pub fn get(&self, name: &str) -> Option<&ParticleEditorValue> {
    self
      .properties
      .iter()
      .find(|property| property.name == name)
      .map(|property| &property.value)
  }

  /// Sets the property by `name`, which keeps its kind.
  ///
  /// # Errors
  ///
  /// Returns an error when the action has no such property, or `value` is of another kind.
  pub fn set(&mut self, name: &str, value: ParticleEditorValue) -> XrfResult {
    let property: &mut ParticleEditorProperty = self
      .properties
      .iter_mut()
      .find(|property| property.name == name)
      .ok_or_else(|| XrfError::new_not_found_error(format!("Particle editor action has no property '{name}'")))?;

    if property.value.get_kind() != value.get_kind() {
      return Err(XrfError::new_invalid_error(format!(
        "Particle editor property '{name}' holds a {:?}, not a {:?}",
        property.value.get_kind(),
        value.get_kind()
      )));
    }

    property.value = value;

    Ok(())
  }

  /// # Errors
  ///
  /// Returns an error when the action has no float property by `name`.
  pub fn get_float(&self, name: &str) -> XrfResult<f32> {
    match self.get_required(name)? {
      ParticleEditorValue::Float(value) => Ok(*value),
      other => Err(Self::new_kind_error(name, ParticleEditorValueKind::Float, other)),
    }
  }

  /// # Errors
  ///
  /// Returns an error when the action has no vector property by `name`.
  pub fn get_vector(&self, name: &str) -> XrfResult<&Vector3d> {
    match self.get_required(name)? {
      ParticleEditorValue::Vector(value) => Ok(value),
      other => Err(Self::new_kind_error(name, ParticleEditorValueKind::Vector, other)),
    }
  }

  /// # Errors
  ///
  /// Returns an error when the action has no domain property by `name`.
  pub fn get_domain(&self, name: &str) -> XrfResult<&ParticleEditorDomain> {
    match self.get_required(name)? {
      ParticleEditorValue::Domain(value) => Ok(value),
      other => Err(Self::new_kind_error(name, ParticleEditorValueKind::Domain, other)),
    }
  }

  /// # Errors
  ///
  /// Returns an error when the action has no bool property by `name`.
  pub fn get_bool(&self, name: &str) -> XrfResult<bool> {
    match self.get_required(name)? {
      ParticleEditorValue::Bool(value) => Ok(*value),
      other => Err(Self::new_kind_error(name, ParticleEditorValueKind::Bool, other)),
    }
  }

  /// # Errors
  ///
  /// Returns an error when the action has no int property by `name`.
  pub fn get_int(&self, name: &str) -> XrfResult<i32> {
    match self.get_required(name)? {
      ParticleEditorValue::Int(value) => Ok(*value),
      other => Err(Self::new_kind_error(name, ParticleEditorValueKind::Int, other)),
    }
  }

  /// The action the engine plays, as the SDK's `Compile` derives it.
  ///
  /// # Errors
  ///
  /// Returns an error when the action lacks a property its kind needs.
  pub fn compile(&self) -> XrfResult<ParticleAction> {
    ParticleEditorActionCompiler::compile(self)
  }

  fn get_required(&self, name: &str) -> XrfResult<&ParticleEditorValue> {
    self.get(name).ok_or_else(|| {
      XrfError::new_not_found_error(format!(
        "Particle editor action '{}' has no property '{name}'",
        self.name
      ))
    })
  }

  fn new_kind_error(name: &str, expected: ParticleEditorValueKind, found: &ParticleEditorValue) -> XrfError {
    XrfError::new_invalid_error(format!(
      "Particle editor property '{name}' holds a {:?}, not a {expected:?}",
      found.get_kind()
    ))
  }

  /// The properties' positions in the order they are stored: by kind, and within a kind by name, as the SDK's maps
  /// iterate their keys.
  fn list_stored_order(&self) -> Vec<usize> {
    let mut order: Vec<usize> = Vec::with_capacity(self.properties.len());

    for kind in ParticleEditorValueKind::STORED_ORDER {
      let mut of_kind: Vec<usize> = (0..self.properties.len())
        .filter(|index| self.properties[*index].value.get_kind() == kind)
        .collect();

      of_kind.sort_by(|first, second| self.properties[*first].name.cmp(&self.properties[*second].name));
      order.extend(of_kind);
    }

    order
  }
}

impl ChunkReadWrite for ParticleEditorAction {
  /// Reads an action as `CPEDef::Load` and `EParticleAction::Load` do: its type, then its version, name, flags and
  /// every property its kind has at that version, in stored order.
  ///
  /// # Errors
  ///
  /// Returns an error for a type no editor action is made of, a version no SDK wrote, or a reader that ends early.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let action_type: ParticleActionType = ParticleActionType::from(reader.read_u32::<T>()?);
    let version: u32 = reader.read_u32::<T>()?;
    let mut action: Self = Self {
      action_type,
      version,
      name: reader.read_w1251_string()?,
      flags: reader.read_u32::<T>()?,
      properties: ParticleEditorSchema::get_defaults(action_type, version)?,
    };

    for index in action.list_stored_order() {
      action.properties[index].value = action.properties[index].value.read_same_kind::<T, _>(reader)?;
    }

    Ok(action)
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_xr::<T, _>(&self.action_type)?;
    writer.write_u32::<T>(self.version)?;
    writer.write_w1251_string(&self.name)?;
    writer.write_u32::<T>(self.flags)?;

    for index in self.list_stored_order() {
      self.properties[index].value.write::<T>(writer)?;
    }

    Ok(())
  }
}

impl ChunkReadWriteList for ParticleEditorAction {
  /// Reads the count and the actions `CPEDef::Load` reads from the editor data chunk.
  fn read_list<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Vec<Self>> {
    let count: u32 = reader.read_u32::<T>()?;
    let mut actions: Vec<Self> =
      reader.new_bounded_vec(count.into(), Self::MIN_SERIALIZED_SIZE, "particle editor actions")?;

    for _ in 0..count {
      actions.push(reader.read_xr::<T, _>()?);
    }

    reader.assert_read("Expect particle editor actions to fill their chunk")?;

    Ok(actions)
  }

  fn write_list<T: ByteOrder>(writer: &mut ChunkWriter, list: &[Self]) -> XrfResult {
    writer.write_u32::<T>(to_format_size(list.len(), "particle editor actions")?)?;

    for action in list {
      writer.write_xr::<T, _>(action)?;
    }

    Ok(())
  }
}
