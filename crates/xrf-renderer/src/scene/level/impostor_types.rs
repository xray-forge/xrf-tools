use std::sync::Arc;

use crate::host::render_level_source::RenderLevelSource;
use crate::host::render_tree_type::RenderTreeType;
use crate::scene::level::loader_answer::take_answer;
use crate::thread::loader_receiver::LoaderReceiver;
use crate::thread::render_workers::RenderWorkers;

/// A level's tree types as its baked impostors need them, the most stood first: read from its source on a loader
/// thread the first time they are asked for, since keying them reads the level's geometry.
#[derive(Default)]
pub struct ImpostorTypes {
  types: Vec<RenderTreeType>,
  reading: Option<LoaderReceiver<Vec<RenderTreeType>>>,
  is_read: bool,
}

impl ImpostorTypes {
  /// Asks for the types where they are neither read nor on their way.
  pub fn read(&mut self, source: &Arc<dyn RenderLevelSource>, workers: &RenderWorkers) {
    if self.is_read || self.reading.is_some() {
      return;
    }

    let (sender, receiver) = LoaderReceiver::channel();
    let source: Arc<dyn RenderLevelSource> = Arc::clone(source);

    workers.spawn(move || {
      let types: Vec<RenderTreeType> = source.read_tree_types().unwrap_or_else(|error| {
        log::warn!("The level's tree types cannot be keyed, so their impostors are not cached: {error}");

        Vec::new()
      });
      // The impostors let go of while they were read answer no one.
      let _ = sender.send(types);
    });

    self.reading = Some(receiver);
  }

  /// Takes the types in once they are read; answers whether they are. A read that died answers none.
  pub fn poll(&mut self) -> bool {
    if self.reading.is_some() {
      if let Some(types) = take_answer(&mut self.reading, "tree types") {
        self.types = types;
      }

      self.is_read = self.reading.is_none();
    }

    self.is_read
  }

  /// Every type read, by the index a sector impostor's type names; none for a level whose types are unknown.
  pub fn list(&self) -> &[RenderTreeType] {
    &self.types
  }

  /// A type's content key, where it is known.
  pub fn get_key(&self, kind: u32) -> Option<u64> {
    self.types.get(kind as usize).map(|kind| kind.key)
  }
}
