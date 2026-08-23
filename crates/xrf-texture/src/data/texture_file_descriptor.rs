use std::cmp::max;
use std::path::PathBuf;

use xrf_error::XrfResult;
use xrf_vfs::XrayLogicalPath;

use crate::constants::DDS_BLOCK_ALIGNMENT;
use crate::data::TextureSpriteDescriptor;

pub struct TextureFileDescriptor {
  pub name: String,
  pub sprites: Vec<TextureSpriteDescriptor>,
}

impl TextureFileDescriptor {
  pub fn new<T>(name: T) -> Self
  where
    T: Into<String>,
  {
    Self {
      name: name.into(),
      sprites: Vec::new(),
    }
  }

  pub fn add_sprite(&mut self, texture: TextureSpriteDescriptor) {
    self.sprites.push(texture);
  }

  /// Converts this description's X-Ray sheet name into a path relative to a trusted host root.
  ///
  /// Texture descriptions use engine separators regardless of where the CLI runs, so filesystem
  /// callers must cross the path boundary through [`XrayLogicalPath`].
  pub fn to_host_relative_path(&self) -> XrfResult<PathBuf> {
    Ok(XrayLogicalPath::new(&self.name)?.to_host_relative_path())
  }

  /// Smallest `DDS_BLOCK_ALIGNMENT`-aligned canvas that holds every described sprite.
  ///
  /// The rounding only ever adds what alignment needs, at most three pixels per axis. A canvas that is
  /// already aligned is returned untouched.
  pub fn get_dimension_boundaries(&self) -> (u32, u32) {
    let mut max_width: u32 = 0;
    let mut max_height: u32 = 0;

    for texture in &self.sprites {
      let (width, height) = texture.get_dimension_boundaries();

      max_width = max(width, max_width);
      max_height = max(height, max_height);
    }

    (
      max_width.next_multiple_of(DDS_BLOCK_ALIGNMENT),
      max_height.next_multiple_of(DDS_BLOCK_ALIGNMENT),
    )
  }
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use crate::data::{TextureFileDescriptor, TextureSpriteDescriptor};

  fn descriptor_of(sprites: &[(u32, u32, u32, u32)]) -> TextureFileDescriptor {
    let mut descriptor: TextureFileDescriptor = TextureFileDescriptor::new(r"ui\ui_actor_weapons");

    for (index, (x, y, w, h)) in sprites.iter().enumerate() {
      descriptor.add_sprite(TextureSpriteDescriptor::new(format!("sprite_{index}"), *x, *y, *w, *h));
    }

    descriptor
  }

  #[test]
  fn bounds_the_furthest_sprite_edges() {
    // The furthest edges are 1023x1020, of which only the width needs rounding.
    assert_eq!(
      descriptor_of(&[(0, 0, 90, 44), (933, 910, 90, 110), (180, 88, 90, 44)]).get_dimension_boundaries(),
      (1024, 1020)
    );
  }

  #[test]
  fn rounds_boundaries_up_to_a_whole_block() {
    for (content, expected) in [
      ((1023, 1020), (1024, 1020)),
      ((1019, 1019), (1020, 1020)),
      ((38, 22), (40, 24)),
    ] {
      assert_eq!(
        descriptor_of(&[(0, 0, content.0, content.1)]).get_dimension_boundaries(),
        expected,
        "Expect {content:?} to round up to {expected:?}"
      );
    }
  }

  #[test]
  fn leaves_an_aligned_canvas_alone() {
    for aligned in [(1024, 1024), (2900, 1000), (980, 1012)] {
      assert_eq!(
        descriptor_of(&[(0, 0, aligned.0, aligned.1)]).get_dimension_boundaries(),
        aligned,
        "Expect an already aligned canvas not to gain a block"
      );
    }
  }

  #[test]
  fn converts_a_nested_sheet_name_to_a_host_path() {
    let descriptor: TextureFileDescriptor = TextureFileDescriptor::new(r"ui\ui_actor_weapons");

    assert_eq!(
      descriptor.to_host_relative_path().expect("valid logical path"),
      PathBuf::from("ui").join("ui_actor_weapons")
    );
  }
}
