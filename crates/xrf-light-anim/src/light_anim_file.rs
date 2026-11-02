use std::fs::File;
use std::path::Path;

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::light_anim_item::LightAnimItem;

/// The colour animation library, `lanims.xr`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LightAnimFile {
  /// Version 0 stores its colours as BGR; the engine swaps them on load and writes 1 back.
  pub version: u16,
  pub items: Vec<LightAnimItem>,
}

impl LightAnimFile {
  /// The library's path under `$game_data$`, the one `ELightAnimLibrary::Load` reads.
  pub const FILE_NAME: &'static str = "lanims.xr";

  pub const VERSION_CHUNK_ID: u32 = 0x0000;
  pub const ITEMS_CHUNK_ID: u32 = 0x0001;

  /// The version below which colours are stored channel-swapped.
  pub const BGR_VERSION: u16 = 0;
  /// The version `ELightAnimLibrary::Save` writes (`LANIM_VERSION`), its colours stored as RGB.
  pub const VERSION: u16 = 1;

  /// Reads a colour animation library from a path.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be opened, or is not a colour animation library this reads.
  pub fn read_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    Self::read_from_file::<T>(File::open(path).map_err(|error| {
      XrfError::new_not_found_error(format!(
        "Light animation library was not read: {}, error: {error}",
        format_path(path.as_ref())
      ))
    })?)
  }

  /// Reads a colour animation library from an open file.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be read, or is not a colour animation library this reads.
  pub fn read_from_file<T: ByteOrder>(file: File) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_file(file)?)
  }

  /// Reads from bytes already in hand, which is how an archived library arrives.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are not a colour animation library this reads.
  pub fn read_from_bytes<T: ByteOrder>(bytes: Vec<u8>) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_vec(bytes)?)
  }

  /// Reads from a chunk reader over any data source.
  ///
  /// # Errors
  ///
  /// Returns an error when an animation cannot be read. A file carrying no version chunk reads as version 0, which
  /// is what `ELightAnimLibrary::Load` does with one.
  pub fn read_from_chunk<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let chunks: Vec<ChunkReader<D>> = reader.read_children()?;

    Ok(Self {
      version: match ChunkReader::find_optional_by_id(&chunks, Self::VERSION_CHUNK_ID) {
        Some(mut chunk) => chunk.read_u16::<T>()?,
        None => Self::BGR_VERSION,
      },
      items: Self::read_items::<T, D>(&mut ChunkReader::find_required_by_id(&chunks, Self::ITEMS_CHUNK_ID)?)?,
    })
  }

  /// Writes the library back in the chunk order `ELightAnimLibrary::Load` reads it in.
  ///
  /// # Errors
  ///
  /// Returns an error when the writer refuses the bytes.
  pub fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    let mut version: ChunkWriter = ChunkWriter::new();

    version.write_u16::<T>(self.version)?;
    version.flush_chunk_into::<T>(&mut writer.buffer, Self::VERSION_CHUNK_ID)?;

    let mut items: ChunkWriter = ChunkWriter::new();

    for (index, item) in self.items.iter().enumerate() {
      let mut chunk: ChunkWriter = ChunkWriter::new();

      item.write::<T>(&mut chunk)?;
      chunk.flush_chunk_into::<T>(&mut items.buffer, index as u32)?;
    }

    items.flush_chunk_into::<T>(&mut writer.buffer, Self::ITEMS_CHUNK_ID)?;

    Ok(())
  }

  /// Reads the animations `ELightAnimLibrary::Load` reaches: chunk 0, 1, 2 and on, stopping at the first id the list
  /// lacks, so an animation numbered past a gap is one the game never has.
  fn read_items<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Vec<LightAnimItem>> {
    let chunks: Vec<ChunkReader<D>> = reader.read_children()?;
    let mut items: Vec<LightAnimItem> = Vec::with_capacity(chunks.len());

    while let Some(mut chunk) = ChunkReader::find_optional_by_id(&chunks, items.len() as u32) {
      items.push(LightAnimItem::read::<T, D>(&mut chunk)?);
    }

    Ok(items)
  }

  /// `ELightAnimLibrary::FindItem`: the first animation named exactly `name`, case included, and none for an empty
  /// name, which every lamp without an animation carries.
  pub fn find_item(&self, name: &str) -> Option<&LightAnimItem> {
    if name.is_empty() {
      return None;
    }

    self.items.iter().find(|item| item.name == name)
  }

  /// [`Self::find_item`], for changing the animation it finds.
  pub fn find_item_mut(&mut self, name: &str) -> Option<&mut LightAnimItem> {
    if name.is_empty() {
      return None;
    }

    self.items.iter_mut().find(|item| item.name == name)
  }

  /// Turns the library into what `ELightAnimLibrary::Load` holds and `Save` writes back: version 1, a version 0
  /// library's colours with red and blue exchanged, and every animation's keys by frame, one a frame.
  pub fn normalize(&mut self) {
    let is_bgr: bool = self.is_bgr();

    for item in &mut self.items {
      item.normalize_keys(is_bgr);
    }

    self.version = Self::VERSION;
  }

  /// `ELightAnimLibrary::AppendItem` without a source: a new animation of the default rate and length, keyed
  /// transparent black at frame 0, added last.
  ///
  /// # Errors
  ///
  /// Returns an error for an empty name, or one the library already holds.
  pub fn append_item(&mut self, name: &str) -> XrfResult<&mut LightAnimItem> {
    self.push_item(LightAnimItem::new(name))
  }

  /// `ELightAnimLibrary::AppendItem` from a source: a copy of the animation named `source` under `name`, added last.
  ///
  /// # Errors
  ///
  /// Returns an error when the library holds no `source`, or `name` is empty or already taken.
  pub fn clone_item(&mut self, source: &str, name: &str) -> XrfResult<&mut LightAnimItem> {
    let mut copy: LightAnimItem = self
      .find_item(source)
      .ok_or_else(|| Self::new_missing_error(source))?
      .clone();

    copy.name = name.to_owned();

    self.push_item(copy)
  }

  /// `ELightAnimLibrary::RenameObject`: gives the animation named `from` the name `to`.
  ///
  /// # Errors
  ///
  /// Returns an error when the library holds no `from`, or `to` is empty or already taken by another animation.
  pub fn rename_item(&mut self, from: &str, to: &str) -> XrfResult {
    if from != to {
      self.require_free_name(to)?;
    }

    self
      .find_item_mut(from)
      .ok_or_else(|| Self::new_missing_error(from))?
      .name = to.to_owned();

    Ok(())
  }

  /// `ELightAnimLibrary::RemoveObject`: takes the animation named `name` out of the library, keeping the others in their
  /// order.
  ///
  /// # Errors
  ///
  /// Returns an error when the library holds no such animation.
  pub fn remove_item(&mut self, name: &str) -> XrfResult<LightAnimItem> {
    let index: usize = self
      .items
      .iter()
      .position(|item| !name.is_empty() && item.name == name)
      .ok_or_else(|| Self::new_missing_error(name))?;

    Ok(self.items.remove(index))
  }

  fn push_item(&mut self, item: LightAnimItem) -> XrfResult<&mut LightAnimItem> {
    self.require_free_name(&item.name)?;
    self.items.push(item);

    Ok(self.items.last_mut().expect("the animation was just added"))
  }

  /// The engine looks an animation up by its exact name and takes the first, so a second of one name is unreachable.
  fn require_free_name(&self, name: &str) -> XrfResult {
    if name.is_empty() {
      return Err(XrfError::new_invalid_error("A light animation needs a name"));
    }

    if self.find_item(name).is_some() {
      return Err(XrfError::new_invalid_error(format!(
        "The light animation library already holds '{name}'"
      )));
    }

    Ok(())
  }

  fn new_missing_error(name: &str) -> XrfError {
    XrfError::new_not_found_error(format!("The light animation library holds no '{name}'"))
  }

  /// Keys across every animation.
  pub fn get_keys_count(&self) -> usize {
    self.items.iter().map(|item| item.keys.len()).sum()
  }

  /// Whether the colours are stored channel-swapped, which the engine corrects on load.
  pub const fn is_bgr(&self) -> bool {
    self.version == Self::BGR_VERSION
  }
}
