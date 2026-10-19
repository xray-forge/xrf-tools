use std::cmp::max;

use xrf_dds::DDS_BLOCK_SIZE;
use xrf_error::XrfResult;
use xrf_extension::XrayExtension;
use xrf_utils::HostRelativePath;
use xrf_vfs::XrayLogicalPath;

use crate::description::TextureSpriteDescriptor;

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

  /// Converts this description's X-Ray sheet name into a path below a host root.
  ///
  /// # Errors
  ///
  /// Returns an error when the sheet name is not a logical path or would leave the root.
  pub fn to_host_relative_path(&self) -> XrfResult<HostRelativePath> {
    HostRelativePath::new(XrayLogicalPath::new(&self.name)?.as_str(), "Texture sheet name")
  }

  /// Where each sprite's own DDS sits below a host root, in sprite order: inside the sheet's directory.
  ///
  /// # Errors
  ///
  /// Returns an error when the sheet name or any sprite id would leave the root.
  pub fn to_sprite_paths(&self) -> XrfResult<Vec<HostRelativePath>> {
    let sheet: HostRelativePath = self.to_host_relative_path()?;

    self
      .sprites
      .iter()
      .map(|sprite| Ok(sheet.join(&sprite.to_host_file_name(XrayExtension::Dds)?)))
      .collect()
  }

  /// Smallest `DDS_BLOCK_SIZE`-aligned canvas that holds every described sprite.
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
      max_width.next_multiple_of(DDS_BLOCK_SIZE),
      max_height.next_multiple_of(DDS_BLOCK_SIZE),
    )
  }
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use crate::description::{TextureFileDescriptor, TextureSpriteDescriptor};

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
      descriptor
        .to_host_relative_path()
        .expect("valid logical path")
        .as_path(),
      PathBuf::from("ui").join("ui_actor_weapons")
    );
  }

  #[test]
  fn places_each_sprite_inside_its_sheet_directory_as_spelled() {
    let mut descriptor: TextureFileDescriptor = TextureFileDescriptor::new(r"ui\ui_icons");

    descriptor.add_sprite(TextureSpriteDescriptor::new("ui_icon_PDA", 0, 0, 4, 4));

    assert_eq!(
      descriptor
        .to_sprite_paths()
        .expect("valid sprite ids")
        .iter()
        .map(|path| path.as_path().to_path_buf())
        .collect::<Vec<PathBuf>>(),
      [PathBuf::from("ui").join("ui_icons").join("ui_icon_PDA.dds")]
    );
  }

  #[test]
  fn refuses_a_sheet_name_with_a_drive() {
    for name in [r"c:\victim", "c:victim", r"ui\sheet:stream"] {
      assert!(
        TextureFileDescriptor::new(name).to_host_relative_path().is_err(),
        "'{name}' must not become a host path"
      );
    }
  }

  #[test]
  fn refuses_a_sprite_id_that_leaves_its_sheet_directory() {
    for id in [
      r"..\..\victim",
      "../../victim",
      r"nested\victim",
      "/rooted",
      r"c:\victim",
      "c:victim",
      "",
      "..",
    ] {
      let mut descriptor: TextureFileDescriptor = TextureFileDescriptor::new(r"ui\ui_icons");

      descriptor.add_sprite(TextureSpriteDescriptor::new(id, 0, 0, 4, 4));

      assert!(
        descriptor.to_sprite_paths().is_err(),
        "sprite id '{id}' must be refused"
      );
    }
  }
}
