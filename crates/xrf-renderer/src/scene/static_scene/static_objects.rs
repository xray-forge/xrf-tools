use std::collections::HashMap;

use xrf_renderer_core::{ProxyHandle, ProxyStore};

use crate::scene::static_scene::static_model_proxy::StaticModelProxy;
use crate::scene::static_scene::static_object_proxy::StaticObjectProxy;

/// The spawned objects standing in a static scene, found by their handle, by the spawned object they are and by the
/// place they stand at.
#[derive(Default)]
pub struct StaticObjects {
  store: ProxyStore<StaticObjectProxy>,
  by_object: HashMap<u32, ProxyHandle<StaticObjectProxy>>,
  by_place: HashMap<u32, ProxyHandle<StaticObjectProxy>>,
}

impl StaticObjects {
  pub fn get(&self, handle: ProxyHandle<StaticObjectProxy>) -> Option<&StaticObjectProxy> {
    self.store.get(handle)
  }

  /// The object standing as a spawned object, where one does.
  pub fn find(&self, object: u32) -> Option<&StaticObjectProxy> {
    self.store.get(*self.by_object.get(&object)?)
  }

  /// The object standing at a place, where one does.
  pub fn find_at(&self, place: u32) -> Option<&StaticObjectProxy> {
    self.store.get(*self.by_place.get(&place)?)
  }

  pub fn contains(&self, object: u32) -> bool {
    self.by_object.contains_key(&object)
  }

  pub fn as_slice(&self) -> &[StaticObjectProxy] {
    self.store.as_slice()
  }

  pub fn iter(&self) -> impl Iterator<Item = (ProxyHandle<StaticObjectProxy>, &StaticObjectProxy)> {
    self.store.iter()
  }

  /// The objects standing as a model.
  pub fn list_standing(&self, model: ProxyHandle<StaticModelProxy>) -> Vec<ProxyHandle<StaticObjectProxy>> {
    self
      .store
      .iter()
      .filter(|(_, object)| object.model == model)
      .map(|(handle, _)| handle)
      .collect()
  }

  /// Puts an object at its handle; answers the one standing as the same spawned object before, which it replaces.
  pub fn insert(
    &mut self,
    handle: ProxyHandle<StaticObjectProxy>,
    object: StaticObjectProxy,
  ) -> Option<ProxyHandle<StaticObjectProxy>> {
    let (index, place): (u32, u32) = (object.object, object.place.get_offset());

    self.store.insert(handle, object);
    self.by_place.insert(place, handle);

    self.by_object.insert(index, handle)
  }

  /// Takes an object out, answering it; none for one already gone.
  pub fn remove(&mut self, handle: ProxyHandle<StaticObjectProxy>) -> Option<StaticObjectProxy> {
    let object: StaticObjectProxy = self.store.remove(handle)?;

    if self.by_object.get(&object.object) == Some(&handle) {
      self.by_object.remove(&object.object);
    }

    self.by_place.remove(&object.place.get_offset());

    Some(object)
  }
}
