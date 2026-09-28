//! A loaded level's own descriptors and textures, read over the shared tree's the way `LoadTHM` and `texture_load` do.

use xrf_shaders::ShaderBlenderClass;
use xrf_shaders::fixtures::ShaderBlenderFixture;
use xrf_thm::{ThmBumpMode, ThmMaterial, ThmTextureFlag, ThmTextureType};
use xrf_vfs::{XrayLogicalPath, XrayMountId, XrayProbe, XrayResolution, XrayVfs};

use crate::fixtures::{FixtureTree, ThmFixture};
use crate::tests::material_probe::{BASE, BUMP, COMPANION, located_path, probe_over, used_bump};
use crate::{
  XrayBumpMode, XrayBumpOutcome, XrayDetailUsage, XrayMaterialBump, XrayMaterialDeclaration, XrayMaterialDescriptor,
  XrayMaterialDetail, XrayMaterialResolver, XraySurfaceDescriptor, XraySurfaceResolver, XrayTextureScope,
};

const LEVEL: &str = "k00_marsh";
const DIRECTORY: &str = "levels\\k00_marsh";
const LEVEL_DESCRIPTOR: &str = "levels\\k00_marsh\\act\\act_stalker.thm";
const SHARED_DESCRIPTOR: &str = "textures\\act\\act_stalker.thm";
const DETAIL: &str = "detail\\detail_grnd_grass";
const LEVEL_DETAIL: &str = "detail\\detail_grnd_asphalt";

fn level_scope() -> XrayTextureScope {
  XrayTextureScope::of_level(XrayLogicalPath::new(DIRECTORY).expect("a level directory"))
}

/// Describes the base texture of `tree` within `scope`.
fn describe_within(tree: &FixtureTree, scope: &XrayTextureScope) -> XrayMaterialDescriptor {
  let mut vfs: XrayVfs = XrayVfs::new();
  let id: XrayMountId = vfs.mount_directory("", tree.root()).expect("tree mounts");

  XrayMaterialResolver::describe_texture(&probe_over(&vfs, id), scope, BASE)
}

fn describe_in_level(tree: &FixtureTree) -> XrayMaterialDescriptor {
  describe_within(tree, &level_scope())
}

fn descriptor_path(descriptor: &XrayMaterialDescriptor) -> Option<&str> {
  descriptor
    .descriptor
    .as_ref()
    .map(|asset| asset.get_logical_path().as_str())
}

/// A shared descriptor declaring no bump, at the SDK's default model, and the pair a level's may declare.
fn shared_flat_tree(case: &str) -> FixtureTree {
  FixtureTree::new(case)
    .with_engine_dummies()
    .with_texture(BASE)
    .with_texture(BUMP)
    .with_texture(COMPANION)
    .with_descriptor(BASE, &ThmFixture::image())
}

#[test]
fn a_level_descriptor_replaces_the_shared_bump_and_material() {
  let descriptor: XrayMaterialDescriptor =
    describe_in_level(&shared_flat_tree("level_override").with_level_descriptor(
      LEVEL,
      BASE,
      &used_bump().with_material(ThmMaterial::PhongMetal, 0.5),
    ));

  assert_eq!(descriptor_path(&descriptor), Some(LEVEL_DESCRIPTOR));
  assert_eq!(
    descriptor.declaration,
    XrayMaterialDeclaration::Declared {
      mode: XrayBumpMode::Use,
      name: BUMP.to_owned(),
    }
  );
  assert_eq!(descriptor.outcome, XrayBumpOutcome::Bumped);
  assert_eq!(descriptor.material, 2.5);
}

#[test]
fn a_level_without_its_own_descriptor_reads_the_shared_one() {
  let descriptor: XrayMaterialDescriptor = describe_in_level(
    &FixtureTree::new("level_fallback")
      .with_texture(BASE)
      .with_texture(BUMP)
      .with_texture(COMPANION)
      .with_descriptor(BASE, &used_bump()),
  );

  assert_eq!(descriptor_path(&descriptor), Some(SHARED_DESCRIPTOR));
  assert_eq!(descriptor.declared_bump_pair(), Some((BUMP, COMPANION)));
}

#[test]
fn a_level_descriptor_with_no_shared_twin_is_read_alone() {
  let descriptor: XrayMaterialDescriptor = describe_in_level(
    &FixtureTree::new("level_alone")
      .with_texture(BASE)
      .with_texture(BUMP)
      .with_texture(COMPANION)
      .with_level_descriptor(LEVEL, BASE, &used_bump()),
  );

  assert_eq!(descriptor_path(&descriptor), Some(LEVEL_DESCRIPTOR));
  assert_eq!(descriptor.outcome, XrayBumpOutcome::Bumped);
}

#[test]
fn a_level_descriptor_switching_its_bump_off_erases_the_shared_bump() {
  // A descriptor the type gate admits assigns `m_spec` whatever its bump says, `tbmNone` included.
  let descriptor: XrayMaterialDescriptor = describe_in_level(
    &FixtureTree::new("level_bump_off")
      .with_texture(BASE)
      .with_texture(BUMP)
      .with_texture(COMPANION)
      .with_descriptor(BASE, &used_bump())
      .with_level_descriptor(LEVEL, BASE, &ThmFixture::image().with_bump(ThmBumpMode::None, "")),
  );

  assert!(
    matches!(descriptor.declaration, XrayMaterialDeclaration::Disabled { .. }),
    "{:?}",
    descriptor.declaration
  );
  assert_eq!(descriptor.outcome, XrayBumpOutcome::Flat);
}

#[test]
fn a_level_descriptor_the_type_gate_skips_leaves_the_shared_one_standing() {
  let descriptor: XrayMaterialDescriptor = describe_in_level(
    &FixtureTree::new("level_skipped")
      .with_texture(BASE)
      .with_texture(BUMP)
      .with_texture(COMPANION)
      .with_descriptor(BASE, &used_bump().with_material(ThmMaterial::PhongMetal, 0.0))
      .with_level_descriptor(
        LEVEL,
        BASE,
        &ThmFixture::image().with_texture_type(ThmTextureType::CubeMap),
      ),
  );

  assert_eq!(descriptor_path(&descriptor), Some(SHARED_DESCRIPTOR));
  assert_eq!(descriptor.declared_bump_pair(), Some((BUMP, COMPANION)));
  assert_eq!(descriptor.material, 2.0);
}

#[test]
fn a_level_descriptor_naming_no_live_detail_keeps_the_shared_detail() {
  // `LoadTHM` replaces the association only for a descriptor naming a detail with a usage flag set.
  let descriptor: XrayMaterialDescriptor = describe_in_level(
    &shared_flat_tree("level_detail_kept")
      .with_descriptor(
        BASE,
        &ThmFixture::image().with_detail(DETAIL, 4.0, &[ThmTextureFlag::DiffuseDetail]),
      )
      .with_level_descriptor(LEVEL, BASE, &used_bump().with_detail(LEVEL_DETAIL, 8.0, &[])),
  );
  let detail: &XrayMaterialDetail = descriptor.detail.as_ref().expect("the shared association survives");

  assert_eq!(descriptor_path(&descriptor), Some(LEVEL_DESCRIPTOR));
  assert_eq!(
    descriptor.outcome,
    XrayBumpOutcome::Bumped,
    "the level's spec still applies"
  );
  assert_eq!(
    (detail.name.as_str(), detail.scale, detail.usage),
    (DETAIL, 4.0, Some(XrayDetailUsage::Diffuse))
  );
}

#[test]
fn a_level_descriptor_naming_a_live_detail_replaces_the_shared_one() {
  let descriptor: XrayMaterialDescriptor = describe_in_level(
    &shared_flat_tree("level_detail_replaced")
      .with_descriptor(
        BASE,
        &ThmFixture::image().with_detail(DETAIL, 4.0, &[ThmTextureFlag::DiffuseDetail]),
      )
      .with_level_descriptor(
        LEVEL,
        BASE,
        &ThmFixture::image().with_detail(LEVEL_DETAIL, 8.0, &[ThmTextureFlag::BumpDetail]),
      ),
  );
  let detail: XrayMaterialDetail = descriptor.detail.expect("the level's association applies");

  assert_eq!(
    (detail.name.as_str(), detail.scale, detail.usage),
    (LEVEL_DETAIL, 8.0, Some(XrayDetailUsage::Bump))
  );
}

#[test]
fn an_unreadable_level_descriptor_is_reported_rather_than_the_shared_one() {
  let descriptor: XrayMaterialDescriptor = describe_in_level(
    &shared_flat_tree("level_unreadable")
      .with_descriptor(
        BASE,
        &used_bump().with_detail(DETAIL, 4.0, &[ThmTextureFlag::DiffuseDetail]),
      )
      .with_unreadable_level_descriptor(LEVEL, BASE),
  );

  assert!(descriptor.is_unreadable());
  assert_eq!(descriptor_path(&descriptor), Some(LEVEL_DESCRIPTOR));
  assert_eq!(descriptor.detail, None);
}

#[test]
fn outside_a_level_a_level_descriptor_is_never_read() {
  let descriptor: XrayMaterialDescriptor = describe_within(
    &shared_flat_tree("level_outside").with_level_descriptor(LEVEL, BASE, &used_bump()),
    &XrayTextureScope::shared(),
  );

  assert_eq!(descriptor_path(&descriptor), Some(SHARED_DESCRIPTOR));
  assert_eq!(descriptor.outcome, XrayBumpOutcome::Flat);
}

#[test]
fn a_level_copy_of_a_bump_pair_is_the_resolution_reported() {
  // `texture_load` searches `$level$` first for a bump too, once `$game_textures$` holds it.
  let descriptor: XrayMaterialDescriptor = describe_in_level(
    &FixtureTree::new("level_bump_copy")
      .with_texture(BASE)
      .with_texture(BUMP)
      .with_texture(COMPANION)
      .with_level_texture(LEVEL, BUMP)
      .with_level_texture(LEVEL, COMPANION)
      .with_descriptor(BASE, &used_bump()),
  );
  let bump: XrayMaterialBump = descriptor.bump.expect("declared");

  assert_eq!(
    located_path(&bump.bump.resolution),
    Some("levels\\k00_marsh\\act\\act_stalker_bump.dds")
  );
  assert_eq!(
    located_path(&bump.companion.resolution),
    Some("levels\\k00_marsh\\act\\act_stalker_bump#.dds")
  );
  assert_eq!(descriptor.outcome, XrayBumpOutcome::Bumped);
}

#[test]
fn a_bump_pair_only_the_level_holds_resolves_to_the_dummy() {
  // `texture_load` asks `$game_textures$` alone whether a `_bump` name exists.
  let descriptor: XrayMaterialDescriptor = describe_in_level(
    &FixtureTree::new("level_bump_level_only")
      .with_engine_dummies()
      .with_texture(BASE)
      .with_level_texture(LEVEL, BUMP)
      .with_level_texture(LEVEL, COMPANION)
      .with_descriptor(BASE, &used_bump()),
  );
  let bump: XrayMaterialBump = descriptor.bump.expect("declared");

  assert!(matches!(bump.bump.resolution, XrayResolution::Substituted { .. }));
  assert_eq!(
    located_path(&bump.bump.resolution),
    Some("textures\\ed\\ed_dummy_bump.dds")
  );
  assert_eq!(
    located_path(&bump.companion.resolution),
    Some("textures\\ed\\ed_dummy_bump#.dds")
  );
  assert_eq!(descriptor.outcome, XrayBumpOutcome::Dummy);
}

#[test]
fn a_bump_named_without_the_marker_resolves_beside_the_level_alone() {
  let descriptor: XrayMaterialDescriptor = describe_in_level(
    &FixtureTree::new("level_bump_unmarked")
      .with_engine_dummies()
      .with_texture(BASE)
      .with_level_texture(LEVEL, "act\\act_stalker_nm")
      .with_level_texture(LEVEL, "act\\act_stalker_nm#")
      .with_descriptor(
        BASE,
        &ThmFixture::image().with_bump(ThmBumpMode::Use, "act\\act_stalker_nm"),
      ),
  );
  let bump: XrayMaterialBump = descriptor.bump.expect("declared");

  assert_eq!(
    located_path(&bump.bump.resolution),
    Some("levels\\k00_marsh\\act\\act_stalker_nm.dds")
  );
  assert_eq!(descriptor.outcome, XrayBumpOutcome::Bumped);
}

#[test]
fn a_texture_resolves_beside_the_level_before_the_shared_tree() {
  let tree: FixtureTree = FixtureTree::new("level_texture_order")
    .with_texture(BASE)
    .with_texture("prop\\prop_fence")
    .with_level_texture(LEVEL, BASE);
  let mut vfs: XrayVfs = XrayVfs::new();
  let id: XrayMountId = vfs.mount_directory("", tree.root()).expect("tree mounts");
  let scope: XrayTextureScope = level_scope();

  let resolve = |reference: &str| -> Option<String> {
    scope
      .resolve_texture(&probe_over(&vfs, id), reference)
      .ok()
      .as_ref()
      .and_then(located_path)
      .map(str::to_owned)
  };

  assert_eq!(
    resolve(BASE).as_deref(),
    Some("levels\\k00_marsh\\act\\act_stalker.dds")
  );
  assert_eq!(
    resolve("prop\\prop_fence").as_deref(),
    Some("textures\\prop\\prop_fence.dds")
  );
}

#[test]
fn a_surface_in_a_level_binds_the_pair_and_model_the_level_descriptor_declares() {
  let tree: FixtureTree = shared_flat_tree("level_surface")
    .with_shader_library(&[ShaderBlenderFixture::of(ShaderBlenderClass::DEFAULT, "default")])
    .with_level_descriptor(LEVEL, BASE, &used_bump().with_material(ThmMaterial::PhongMetal, 0.5));
  let mut vfs: XrayVfs = XrayVfs::new();
  let id: XrayMountId = vfs.mount_directory("", tree.root()).expect("tree mounts");
  let probe: XrayProbe<'_> = probe_over(&vfs, id);

  let within: XraySurfaceDescriptor =
    XraySurfaceResolver::open(&probe, level_scope()).describe("default", &[BASE.to_owned()]);
  let outside: XraySurfaceDescriptor =
    XraySurfaceResolver::open(&probe, XrayTextureScope::shared()).describe("default", &[BASE.to_owned()]);

  assert_eq!(
    within.bump.as_ref().map(|bump| bump.bump.reference.as_str()),
    Some(BUMP)
  );
  assert_eq!(within.material, 2.5);
  assert_eq!(outside.bump, None, "the shared descriptor declares no bump");
  assert_eq!(outside.material, XrayMaterialDescriptor::DEFAULT_MATERIAL);
}
