/// How a skin layout the vertex format tag names stores one vertex: its size, and the bones it links.
pub(crate) struct OgfSkinLayout {
  pub(crate) vertex_size: usize,
  pub(crate) links: usize,
}

impl OgfSkinLayout {
  const VERTEX_FORMAT_1_LINK: u32 = 0x12071980;
  const VERTEX_FORMAT_2_LINK: u32 = 2 * Self::VERTEX_FORMAT_1_LINK;
  const VERTEX_FORMAT_3_LINK: u32 = 4 * Self::VERTEX_FORMAT_1_LINK;
  const VERTEX_FORMAT_4_LINK: u32 = 5 * Self::VERTEX_FORMAT_1_LINK;

  /// The layout `vertex_format` names, in its tagged or its plain link count form, or `None` for no skin layout.
  pub(crate) fn of(vertex_format: u32) -> Option<Self> {
    match vertex_format {
      Self::VERTEX_FORMAT_1_LINK | 1 => Some(Self {
        vertex_size: 60,
        links: 1,
      }),
      Self::VERTEX_FORMAT_2_LINK | 2 => Some(Self {
        vertex_size: 64,
        links: 2,
      }),
      Self::VERTEX_FORMAT_3_LINK | 3 => Some(Self {
        vertex_size: 70,
        links: 3,
      }),
      Self::VERTEX_FORMAT_4_LINK | 4 => Some(Self {
        vertex_size: 76,
        links: 4,
      }),
      _ => None,
    }
  }
}
