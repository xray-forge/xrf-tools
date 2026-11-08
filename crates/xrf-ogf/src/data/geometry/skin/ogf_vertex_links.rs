use std::fmt;
use std::ops::{Deref, DerefMut};

use crate::data::geometry::skin::ogf_vertex_link::OgfVertexLink;

/// The bones one skinned vertex is linked to, held inline.
///
/// The engine's skin layouts, `vertBoned1W` to `vertBoned4W`, store one to four links, so the links never need a heap
/// allocation of their own: a skinned visual holds one of these per vertex, and the corpus millions of vertices.
#[derive(Clone, Copy, Default)]
pub struct OgfVertexLinks {
  links: [OgfVertexLink; Self::CAPACITY],
  count: u8,
}

impl OgfVertexLinks {
  /// Links a vertex can carry, as many as the widest skin layout stores.
  pub const CAPACITY: usize = 4;

  /// The links of `slice`, at most [`Self::CAPACITY`] of them.
  ///
  /// # Panics
  ///
  /// When `slice` holds more than [`Self::CAPACITY`] links, which no skin layout stores.
  pub fn from_slice(slice: &[OgfVertexLink]) -> Self {
    assert!(
      slice.len() <= Self::CAPACITY,
      "A vertex carries at most {} links, got {}",
      Self::CAPACITY,
      slice.len()
    );

    let mut links: Self = Self::default();

    links.links[..slice.len()].copy_from_slice(slice);
    links.count = u8::try_from(slice.len()).expect("a count of at most four fits a byte");

    links
  }
}

impl<const N: usize> From<[OgfVertexLink; N]> for OgfVertexLinks {
  fn from(links: [OgfVertexLink; N]) -> Self {
    const { assert!(N <= Self::CAPACITY, "A vertex carries at most four links") };

    Self::from_slice(&links)
  }
}

impl Deref for OgfVertexLinks {
  type Target = [OgfVertexLink];

  fn deref(&self) -> &[OgfVertexLink] {
    &self.links[..usize::from(self.count)]
  }
}

impl DerefMut for OgfVertexLinks {
  fn deref_mut(&mut self) -> &mut [OgfVertexLink] {
    &mut self.links[..usize::from(self.count)]
  }
}

impl<'a> IntoIterator for &'a OgfVertexLinks {
  type Item = &'a OgfVertexLink;
  type IntoIter = std::slice::Iter<'a, OgfVertexLink>;

  fn into_iter(self) -> Self::IntoIter {
    self.iter()
  }
}

/// Equal when the links held are, whatever the unused slots contain.
impl PartialEq for OgfVertexLinks {
  fn eq(&self, other: &Self) -> bool {
    **self == **other
  }
}

/// The links held, as a list.
impl fmt::Debug for OgfVertexLinks {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter.debug_list().entries(self.iter()).finish()
  }
}

#[cfg(test)]
mod tests {
  use crate::data::geometry::skin::ogf_vertex_link::OgfVertexLink;
  use crate::data::geometry::skin::ogf_vertex_links::OgfVertexLinks;

  #[test]
  fn holds_only_the_links_it_was_given() {
    let links: OgfVertexLinks = OgfVertexLinks::from([
      OgfVertexLink { bone: 3, weight: 0.75 },
      OgfVertexLink { bone: 9, weight: 0.25 },
    ]);

    assert_eq!(links.len(), 2);
    assert_eq!(links.iter().map(|it| it.bone).collect::<Vec<u16>>(), vec![3, 9]);
    assert!(OgfVertexLinks::default().is_empty());
  }

  #[test]
  fn compares_the_links_held_and_not_the_unused_slots() {
    let mut first: OgfVertexLinks = OgfVertexLinks::from([OgfVertexLink { bone: 1, weight: 1.0 }; 2]);

    first.links[3].bone = 7;

    assert_eq!(first, OgfVertexLinks::from([OgfVertexLink { bone: 1, weight: 1.0 }; 2]));
    assert_ne!(first, OgfVertexLinks::from([OgfVertexLink { bone: 1, weight: 1.0 }]));
  }

  #[test]
  #[should_panic(expected = "A vertex carries at most 4 links, got 5")]
  fn refuses_more_links_than_any_layout_stores() {
    OgfVertexLinks::from_slice(&[OgfVertexLink::default(); 5]);
  }
}
