//! The vocabulary every textures test is written in: the trees it mounts, the scratch it writes to, and the pictures
//! it measures against.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use xrf_app_core::assets::FileStamp;
use xrf_chunk::XRayByteOrder;
use xrf_dds::{DdsEncoding, DdsMipChain, DdsMipmaps, ImageFormat, Quality, Rgba, RgbaImage};
use xrf_material::fixtures::{FixtureTree, ThmFixture};
use xrf_test_utils::GeneratedResources;
use xrf_thm::{ThmBumpMode, ThmFile};
use xrf_vfs::{
  XrayAsset, XrayAssetContainer, XrayAssetType, XrayLogicalPath, XrayLookupScope, XrayMountId, XrayMountMode,
  XrayRoots, XraySearch, XrayVfs,
};

use crate::browse::{TextureCatalog, TextureCatalogMode, TextureEntry, TextureMaterialSummary};
use crate::describe::TextureSource;
use crate::edit::TextureSaveTarget;

/// One texture and the two halves of the pair it declares, which most of these start from.
pub const BASE: &str = "ston\\ston_beton05";
pub const BUMP: &str = "ston\\ston_beton05_bump";
pub const COMPANION: &str = "ston\\ston_beton05_bump#";

/// One mounted tree and a search naming it.
pub fn mount(tree: &FixtureTree) -> (XrayVfs, XrayMountId) {
  let mut vfs: XrayVfs = XrayVfs::new();
  let id: XrayMountId = vfs.mount_directory("", tree.root()).expect("tree mounts");

  (vfs, id)
}

pub fn search_over(vfs: impl Into<Arc<XrayVfs>>, id: XrayMountId) -> XraySearch {
  XraySearch::new(vfs).with_step("tree", XrayLookupScope::only([id]))
}

pub fn roots_of(tree: &FixtureTree) -> XrayRoots {
  XrayRoots::one(tree.root().to_path_buf(), XrayMountMode::Directory)
}

pub fn catalog(tree: &FixtureTree) -> TextureCatalog {
  let (vfs, id) = mount(tree);

  TextureCatalog::list(&search_over(vfs, id), roots_of(tree), TextureCatalogMode::Roots)
}

/// The sweep as the command runs it: over every descriptor the search lists.
pub fn sweep(tree: &FixtureTree) -> Vec<TextureMaterialSummary> {
  let (vfs, id) = mount(tree);
  let search: XraySearch = search_over(vfs, id);

  TextureMaterialSummary::sweep(&search, &search.list_assets(XrayAssetType::Thm))
}

pub fn entry<'a>(catalog: &'a TextureCatalog, reference: &str) -> &'a TextureEntry {
  catalog
    .entries
    .iter()
    .find(|entry| entry.reference == reference)
    .unwrap_or_else(|| panic!("catalog lists '{reference}'"))
}

pub fn summary<'a>(summaries: &'a [TextureMaterialSummary], reference: &str) -> &'a TextureMaterialSummary {
  summaries
    .iter()
    .find(|summary| summary.reference == reference)
    .unwrap_or_else(|| panic!("sweep describes '{reference}'"))
}

/// A tree with a declared pair that resolves, which is the shape most of these start from.
pub fn bumped_tree(case: &str) -> FixtureTree {
  FixtureTree::new(&format!("textures_{case}"))
    .with_texture(BASE)
    .with_texture(BUMP)
    .with_texture(COMPANION)
    .with_descriptor(BASE, &ThmFixture::image().with_bump(ThmBumpMode::Use, BUMP))
}

/// The fixture tree made an X-Ray root the VFS implies: it holds textures already, so a `meshes` directory completes it.
pub fn implied_root_tree(case: &str) -> FixtureTree {
  let tree: FixtureTree = FixtureTree::new(&format!("textures_{case}")).with_texture(BASE);

  fs::create_dir_all(tree.root().join("meshes")).expect("meshes directory");

  tree
}

pub fn file_source(path: PathBuf) -> TextureSource {
  TextureSource::File {
    path: path.display().to_string(),
  }
}

/// A scratch directory of its own per case, so a save that writes files cannot reach another's.
pub fn editing_directory(case: &str) -> PathBuf {
  to_scratch_directory("textures_editing", case)
}

/// A scratch directory outside every X-Ray root, which is the case a standalone description exists for.
pub fn loose_directory(case: &str) -> PathBuf {
  to_scratch_directory("textures_standalone", case)
}

/// The target a save carries for a file, stamped as the editor would have stamped it.
pub fn target(path: &Path) -> TextureSaveTarget {
  TextureSaveTarget {
    path: path.display().to_string(),
    expected: FileStamp::read(path).expect("stamp is readable"),
  }
}

pub fn read_descriptor(path: &Path) -> ThmFile {
  ThmFile::read_from_path::<XRayByteOrder, _>(&path).expect("written descriptor parses")
}

/// A loose asset at `path`, as the VFS would have reported it.
pub fn loose_asset(root: &Path, relative: &str) -> XrayAsset {
  XrayAsset::new(
    XrayLogicalPath::new(relative).expect("logical path"),
    XrayAssetContainer::Directory {
      root: root.to_path_buf(),
      relative_path: PathBuf::from(relative.replace('\\', "/")),
    },
  )
}

/// A picture with detail in every channel and a half-transparent side, so a lossy candidate has something to lose.
pub fn source_image(size: u32) -> RgbaImage {
  RgbaImage::from_fn(size, size, |x, y| {
    Rgba([
      (x * 7 + y * 13) as u8,
      (x * 31 + y * 3) as u8,
      (x * 17) as u8,
      if x < size / 2 { 0 } else { u8::MAX },
    ])
  })
}

/// A real uncompressed dds, so a standalone description has a header to read rather than a placeholder.
pub fn to_dds_bytes(size: u32) -> Vec<u8> {
  let base: RgbaImage = RgbaImage::from_fn(size, size, |x, y| Rgba([(x * 8) as u8, (y * 8) as u8, 0, u8::MAX]));

  DdsEncoding::new(ImageFormat::Rgba8Unorm, Quality::Fast)
    .encode(&DdsMipChain::build(&base, DdsMipmaps::Disabled).expect("chain"))
    .expect("encode")
    .write_to_bytes()
    .expect("bytes")
}

/// A directory this run owns, emptied on the way in so a case starts from nothing whatever the last run left.
fn to_scratch_directory(prefix: &str, case: &str) -> PathBuf {
  let root: PathBuf = GeneratedResources::path(&format!("{prefix}/{case}"));

  let _ = fs::remove_dir_all(&root);

  fs::create_dir_all(&root).expect("case directory");

  root
}
