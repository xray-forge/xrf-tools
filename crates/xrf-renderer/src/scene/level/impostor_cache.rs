use std::sync::Arc;
use std::sync::mpsc::Sender;

use crate::frame::gpu_readback::GpuReadback;
use crate::host::render_cache_store::RenderCacheStore;
use crate::scene::level::impostor_atlas::ImpostorAtlas;
use crate::scene::level::impostor_scratch::ImpostorBlockLevel;
use crate::thread::loader_receiver::LoaderReceiver;
use crate::thread::render_workers::RenderWorkers;

/// What a read of the cache came back with for a tree type: the atlas size asked for, and every level's blocks of the
/// three maps, unpadded, where a whole file of that size was kept.
pub struct ImpostorLoad {
  pub kind: u32,
  pub size: u32,
  pub blocks: Option<Vec<u8>>,
}

/// A bake's blocks on their way back from the GPU, and the name and atlas size they are kept under once back.
struct ImpostorSave {
  readback: GpuReadback,
  entry: Option<(String, u32)>,
}

/// Baked impostors kept between runs and shared between levels, through the renderer's cache store: a file a tree type
/// and atlas size, named by the type's content key and the bake's version, read and written on the loader threads.
///
/// A file holds a header (`MAGIC`, the bake's version, the atlas's texels a side and its levels), every level's blocks
/// of the three maps one after another, unpadded, and a CRC-32 of all before it.
pub struct ImpostorCache {
  store: Option<Arc<dyn RenderCacheStore>>,
  workers: RenderWorkers,
  /// The bake's version, which a file is named and checked by: a bake drawing or packing otherwise reads none of the
  /// files an earlier one kept.
  version: u32,
  loads: (Sender<ImpostorLoad>, LoaderReceiver<ImpostorLoad>),
  saves: Vec<ImpostorSave>,
}

impl ImpostorCache {
  const MAGIC: [u8; 4] = *b"XRFI";

  /// Bytes of a file's header: the magic, the version, the size and the levels.
  const HEADER: usize = 16;

  pub fn new(store: Option<Arc<dyn RenderCacheStore>>, workers: RenderWorkers, version: u32) -> Self {
    Self {
      store,
      workers,
      version,
      loads: LoaderReceiver::channel(),
      saves: Vec::new(),
    }
  }

  /// Whether a store keeps what it is given, so a type is asked of it before it bakes.
  pub fn is_kept(&self) -> bool {
    self.store.is_some()
  }

  /// The name a type's atlases of a size are kept under: its content key, the size and the bake's version.
  fn get_name(&self, key: u64, size: u32) -> String {
    format!("impostors/{key:016x}-{size}-v{}.bin", self.version)
  }

  /// Asks for the atlases of a size of the type `kind`, keyed `key`, read on a loader thread; `take_loads` answers.
  pub fn ask(&self, (kind, key): (u32, u64), size: u32) {
    let Some(store) = self.store.clone() else {
      return;
    };
    let answers: Sender<ImpostorLoad> = self.loads.0.clone();
    let (name, version): (String, u32) = (self.get_name(key, size), self.version);

    self.workers.spawn(move || {
      let blocks: Option<Vec<u8>> = store
        .read_cached(&name)
        .and_then(|bytes| Self::decode(&bytes, size, version).map(<[u8]>::to_vec));
      // The cache let go of while it read answers no one.
      let _ = answers.send(ImpostorLoad { kind, size, blocks });
    });
  }

  /// What came back of the reads asked for since the last call.
  pub fn take_loads(&mut self) -> impl Iterator<Item = ImpostorLoad> + '_ {
    self.loads.1.try_iter()
  }

  /// Readies a copy of a bake's block buffer, `bytes` long, to be kept as the atlases of a size of the type keyed `key`
  /// once read back: into a free readback as long, or a new one. Answers the copy's index, marked recorded, whose
  /// buffer the bake copies into; none without a store.
  pub fn prepare_save(&mut self, device: &wgpu::Device, (key, size): (u64, u32), bytes: u64) -> Option<usize> {
    self.store.as_ref()?;

    let name: String = self.get_name(key, size);

    let index: usize = match self
      .saves
      .iter()
      .position(|save| save.entry.is_none() && save.readback.is_free() && save.readback.get_buffer().size() >= bytes)
    {
      Some(index) => index,
      None => {
        self.saves.push(ImpostorSave {
          readback: GpuReadback::new(device, "impostor save", bytes),
          entry: None,
        });

        self.saves.len() - 1
      }
    };
    let save: &mut ImpostorSave = &mut self.saves[index];

    save.entry = Some((name, size));
    save.readback.mark_recorded();

    Some(index)
  }

  /// The buffer a copy `prepare_save` readied goes into.
  pub fn get_save_buffer(&self, index: usize) -> &wgpu::Buffer {
    self.saves[index].readback.get_buffer()
  }

  /// Asks for the copies recorded with the frame just submitted to be read back.
  pub fn request(&self) {
    for save in &self.saves {
      save.readback.request();
    }
  }

  /// Drops the copies of a frame that was never submitted.
  pub fn abandon(&mut self) {
    for save in &mut self.saves {
      if save.readback.is_recorded() {
        save.readback.abandon();
        save.entry = None;
      }
    }
  }

  /// Lets go of the readbacks no copy is on its way in, once nothing is left to bake.
  pub fn release_saves(&mut self) {
    self.saves.retain(|save| save.entry.is_some());
  }

  /// Writes each copy back since the last call on a loader thread, unpadded and checked.
  pub fn write_saves(&mut self) {
    let Some(store) = &self.store else {
      return;
    };

    for save in &mut self.saves {
      let Some((name, size)) = save.entry.as_ref().filter(|_| save.readback.is_back()) else {
        continue;
      };
      let (name, size): (String, u32) = (name.clone(), *size);
      let levels: u32 = ImpostorAtlas::get_levels(ImpostorAtlas::get_frame(size));
      let (layouts, block_count): (Vec<ImpostorBlockLevel>, u32) = ImpostorBlockLevel::list(size, levels);
      let padded: Option<Vec<u8>> = match save
        .readback
        .take(|bytes| bytes[..block_count as usize * ImpostorAtlas::BLOCK_BYTES as usize].to_vec())
      {
        Some(Ok(padded)) => Some(padded),
        Some(Err(error)) => {
          log::warn!("A baked impostor cannot be read back to be kept: {error}");

          None
        }
        None => continue,
      };

      save.entry = None;

      if let Some(padded) = padded {
        let store: Arc<dyn RenderCacheStore> = Arc::clone(store);
        let version: u32 = self.version;

        self.workers.spawn(move || {
          store.write_cached(&name, &Self::encode(&padded, &layouts, (size, levels, version)));
        });
      }
    }
  }

  /// Every level's blocks of every map, `padded` as the block buffer lays them, unpadded behind a header, with the
  /// checksum after.
  fn encode(padded: &[u8], layouts: &[ImpostorBlockLevel], (size, levels, version): (u32, u32, u32)) -> Vec<u8> {
    let block: usize = ImpostorAtlas::BLOCK_BYTES as usize;
    let mut bytes: Vec<u8> = Vec::with_capacity(padded.len() + Self::HEADER + 4);

    bytes.extend_from_slice(&Self::MAGIC);

    for value in [version, size, levels] {
      bytes.extend_from_slice(&value.to_le_bytes());
    }

    for layout in layouts {
      for start in layout.starts {
        for row in 0..layout.blocks {
          let at: usize = (start + row * layout.row) as usize * block;

          bytes.extend_from_slice(&padded[at..at + layout.blocks as usize * block]);
        }
      }
    }

    let checksum: u32 = crc32fast::hash(&bytes);

    bytes.extend_from_slice(&checksum.to_le_bytes());
    bytes
  }

  /// The blocks a file holds, where it is whole and of the atlas size and version asked for.
  fn decode(bytes: &[u8], size: u32, version: u32) -> Option<&[u8]> {
    let (body, checksum): (&[u8], &[u8; 4]) = bytes.split_last_chunk::<4>()?;
    let (header, blocks): (&[u8; Self::HEADER], &[u8]) = body.split_first_chunk::<{ Self::HEADER }>()?;
    let read = |at: usize| u32::from_le_bytes([header[at], header[at + 1], header[at + 2], header[at + 3]]);
    let levels: u32 = ImpostorAtlas::get_levels(ImpostorAtlas::get_frame(size));

    let is_whole: bool = header[..4] == Self::MAGIC
      && [read(4), read(8), read(12)] == [version, size, levels]
      && blocks.len() as u64 == ImpostorAtlas::get_bytes(size, levels)
      && u32::from_le_bytes(*checksum) == crc32fast::hash(body);

    is_whole.then_some(blocks)
  }
}
