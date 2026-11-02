use std::path::PathBuf;
use std::sync::Arc;

use crate::vfs::tests::fixtures::{FakeArchiveSource, directory};
use crate::{
  XrayAsset, XrayAssetType, XrayCachePolicy, XrayInput, XrayInputAnswer, XrayInputKind, XrayInputRecord,
  XrayLogicalPath, XrayLookupScope, XrayMountId, XrayRevision, XraySearch, XraySearchKey, XrayVfs,
};

/// A loose tree searched before an archive, returned with both mounts' ids.
fn loose_over_archive(case: &str, loose: &[&str], archived: &[&str]) -> (Arc<XrayVfs>, XrayMountId, XrayMountId) {
  let root: PathBuf = directory(&format!("recording_{case}"), loose);
  let mut vfs: XrayVfs = XrayVfs::new().with_cache_policy(XrayCachePolicy::none().with(XrayAssetType::Ltx));
  let loose_id: XrayMountId = vfs.mount_directory("", &root).expect("loose root mounts");
  let archive_id: XrayMountId = vfs
    .mount("", Box::new(FakeArchiveSource::new("archive", archived)))
    .expect("archive mounts");

  (Arc::new(vfs), loose_id, archive_id)
}

fn search_of(vfs: &Arc<XrayVfs>, loose_id: XrayMountId, archive_id: XrayMountId) -> XraySearch {
  XraySearch::new(Arc::clone(vfs))
    .with_step("loose", XrayLookupScope::only([loose_id]))
    .with_step("archive", XrayLookupScope::only([archive_id]))
}

fn file(search: &XraySearch, path: &str) -> XrayInput {
  XrayInput::new(
    XraySearchKey::of(search.get_steps()),
    XrayInputKind::File(XrayLogicalPath::new(path).expect("test path is valid")),
  )
}

fn listing(search: &XraySearch, directory: Option<&str>) -> XrayInput {
  XrayInput::new(
    XraySearchKey::of(search.get_steps()),
    XrayInputKind::Listing(directory.map(|directory| XrayLogicalPath::new(directory).expect("test path is valid"))),
  )
}

fn found(source: XrayMountId) -> Option<XrayInputAnswer> {
  Some(XrayInputAnswer {
    source: Some(source),
    revision: XrayRevision::MOUNTED,
  })
}

#[test]
fn records_a_resolved_reference_with_the_source_answering_it() {
  let (vfs, loose_id, archive_id) = loose_over_archive(
    "resolved",
    &["textures/wpn/wpn_ak74.dds"],
    &["textures/wpn/wpn_abakan.dds"],
  );
  let record: XrayInputRecord = XrayInputRecord::new();
  let search: XraySearch = search_of(&vfs, loose_id, archive_id).with_recording(&record);

  search
    .resolve(XrayAssetType::Dds, "wpn\\wpn_ak74")
    .expect("lookup succeeds");
  search
    .resolve(XrayAssetType::Dds, "wpn\\wpn_abakan")
    .expect("lookup succeeds");

  assert_eq!(record.len(), 2);
  assert_eq!(
    record.get(&file(&search, "textures\\wpn\\wpn_ak74.dds")),
    found(loose_id)
  );
  assert_eq!(
    record.get(&file(&search, "textures\\wpn\\wpn_abakan.dds")),
    found(archive_id)
  );
}

#[test]
fn records_a_lookup_that_finds_nothing_and_its_fallback() {
  let (vfs, loose_id, archive_id) = loose_over_archive("missing", &["textures/ed/ed_not_existing_texture.dds"], &[]);
  let record: XrayInputRecord = XrayInputRecord::new();
  let search: XraySearch = search_of(&vfs, loose_id, archive_id).with_recording(&record);

  search.find("textures\\absent.dds").expect("lookup succeeds");
  search
    .resolve_with_fallback(XrayAssetType::Dds, "wpn\\absent", "ed\\ed_not_existing_texture")
    .expect("lookup succeeds");

  // Nothing answering is an answer too: a file created there later is what changes it.
  assert_eq!(
    record.get(&file(&search, "textures\\absent.dds")),
    Some(XrayInputAnswer::default())
  );
  assert_eq!(
    record.get(&file(&search, "textures\\wpn\\absent.dds")),
    Some(XrayInputAnswer::default())
  );
  assert_eq!(
    record.get(&file(&search, "textures\\ed\\ed_not_existing_texture.dds")),
    found(loose_id)
  );
}

#[test]
fn records_listings_by_the_directory_they_list() {
  let (vfs, loose_id, archive_id) = loose_over_archive(
    "listings",
    &["textures/wpn/wpn_ak74.dds"],
    &["meshes/wpn/wpn_ak74.omf", "meshes/wpn/wpn_abakan.omf"],
  );
  let record: XrayInputRecord = XrayInputRecord::new();
  let search: XraySearch = search_of(&vfs, loose_id, archive_id).with_recording(&record);

  search.list_children("textures\\wpn").expect("listing succeeds");
  search.list_assets(XrayAssetType::Dds);
  search.resolve(XrayAssetType::Omf, "wpn\\wpn_*").expect("mask resolves");

  let listed: Option<XrayInputAnswer> = Some(XrayInputAnswer::default());

  assert_eq!(record.len(), 3);
  assert_eq!(record.get(&listing(&search, Some("textures\\wpn"))), listed);
  assert_eq!(record.get(&listing(&search, None)), listed);
  assert_eq!(record.get(&listing(&search, Some("meshes"))), listed);
}

#[test]
fn records_a_listing_of_a_prefixed_search_by_the_subtree_it_covers() {
  // A whole listing of a search narrowed to `configs` changes only when something below `configs` does, so a texture
  // created elsewhere must not invalidate it; several prefixes share their longest common directory.
  let (vfs, loose_id, archive_id) = loose_over_archive(
    "prefixed_listing",
    &["configs/weapons/w_ak74.ltx", "configs/system.ltx"],
    &["configs/weapons/w_pm.ltx"],
  );
  let record: XrayInputRecord = XrayInputRecord::new();
  let narrowed: XraySearch = search_of(&vfs, loose_id, archive_id)
    .with_prefix("configs")
    .expect("prefix is valid")
    .with_recording(&record);

  narrowed.list_files();
  narrowed.list_entries();

  assert_eq!(record.len(), 1, "both listings are the one input");
  assert!(record.get(&listing(&narrowed, Some("configs"))).is_some());

  let record: XrayInputRecord = XrayInputRecord::new();
  let split: XraySearch = XraySearch::new(Arc::clone(&vfs))
    .with_step(
      "weapons",
      XrayLookupScope::only([loose_id])
        .with_prefix("configs\\weapons")
        .expect("prefix"),
    )
    .with_step(
      "system",
      XrayLookupScope::only([archive_id])
        .with_prefix("configs\\system.ltx")
        .expect("prefix"),
    )
    .with_recording(&record);

  split.list_assets(XrayAssetType::Ltx);

  assert!(
    record.get(&listing(&split, Some("configs"))).is_some(),
    "two prefixes record the directory both sit under"
  );

  let record: XrayInputRecord = XrayInputRecord::new();
  let unnarrowed: XraySearch = XraySearch::new(Arc::clone(&vfs))
    .with_step(
      "configs",
      XrayLookupScope::only([loose_id])
        .with_prefix("configs")
        .expect("prefix"),
    )
    .with_step("everything", XrayLookupScope::only([archive_id]))
    .with_recording(&record);

  unnarrowed.list_shadowing_entries();

  assert!(
    record.get(&listing(&unnarrowed, None)).is_some(),
    "a step with no prefix lists the whole root"
  );
}

#[test]
fn records_a_read_served_from_the_cache() {
  let (vfs, loose_id, archive_id) = loose_over_archive("cached", &["configs/system.ltx"], &[]);
  let search: XraySearch = search_of(&vfs, loose_id, archive_id);
  let asset: XrayAsset = search
    .find("configs\\system.ltx")
    .expect("lookup succeeds")
    .expect("the file is found");
  let parse = |bytes: Vec<u8>| Ok(bytes.len());

  search.read_parsed(&asset, parse).expect("first read parses");

  let record: XrayInputRecord = XrayInputRecord::new();
  let recording: XraySearch = search.clone().with_recording(&record);

  recording.read_parsed(&asset, parse).expect("second read is served");

  assert_eq!(vfs.get_cache_stats().hits, 1, "the second read is a hit");
  assert_eq!(record.get(&file(&recording, "configs\\system.ltx")), found(loose_id));
}

#[test]
fn records_each_input_once_with_its_first_answer() {
  let (vfs, loose_id, archive_id) = loose_over_archive("once", &["configs/system.ltx"], &[]);
  let record: XrayInputRecord = XrayInputRecord::new();
  let search: XraySearch = search_of(&vfs, loose_id, archive_id).with_recording(&record);

  let asset: XrayAsset = search
    .find("configs\\system.ltx")
    .expect("lookup succeeds")
    .expect("the file is found");

  search.read_asset(&asset).expect("the file reads");
  search.read("configs\\system.ltx").expect("the path reads");
  search.find("Configs/System.ltx").expect("lookup succeeds");

  assert_eq!(record.len(), 1, "a path found, read and found again is one input");
}

#[test]
fn files_an_input_under_the_search_that_resolved_it() {
  let (vfs, loose_id, archive_id) = loose_over_archive("search", &["configs/system.ltx"], &["configs/system.ltx"]);
  let record: XrayInputRecord = XrayInputRecord::new();
  let forward: XraySearch = search_of(&vfs, loose_id, archive_id).with_recording(&record);
  let backward: XraySearch = XraySearch::new(Arc::clone(&vfs))
    .with_step("archive", XrayLookupScope::only([archive_id]))
    .with_step("loose", XrayLookupScope::only([loose_id]))
    .with_recording(&record);

  forward.find("configs\\system.ltx").expect("lookup succeeds");
  backward.find("configs\\system.ltx").expect("lookup succeeds");

  assert_eq!(record.len(), 2, "one path searched two ways is two inputs");
  assert_eq!(record.get(&file(&forward, "configs\\system.ltx")), found(loose_id));
  assert_eq!(record.get(&file(&backward, "configs\\system.ltx")), found(archive_id));
}

#[test]
fn files_inputs_under_steps_added_after_recording_began() {
  let (vfs, loose_id, archive_id) = loose_over_archive("late_step", &[], &["configs/system.ltx"]);
  let record: XrayInputRecord = XrayInputRecord::new();
  let search: XraySearch = XraySearch::new(vfs)
    .with_step("loose", XrayLookupScope::only([loose_id]))
    .with_recording(&record)
    .with_step("archive", XrayLookupScope::only([archive_id]));

  search.find("configs\\system.ltx").expect("lookup succeeds");

  assert_eq!(record.get(&file(&search, "configs\\system.ltx")), found(archive_id));
}

#[test]
fn a_plain_search_records_nothing() {
  let (vfs, loose_id, archive_id) = loose_over_archive("plain", &["configs/system.ltx"], &[]);
  let record: XrayInputRecord = XrayInputRecord::new();
  let plain: XraySearch = search_of(&vfs, loose_id, archive_id);
  let recording: XraySearch = plain.clone().with_recording(&record);

  plain.find("configs\\system.ltx").expect("lookup succeeds");
  plain.record_inputs(&record);

  assert!(
    plain.get_record().is_none(),
    "recording leaves the search it was cloned from plain"
  );
  assert!(recording.get_record().is_some());
  assert!(record.is_empty());
}

#[test]
fn a_clone_of_a_recording_search_records_into_the_same_record() {
  let (vfs, loose_id, archive_id) = loose_over_archive("clone", &["configs/system.ltx"], &["configs/game.ltx"]);
  let record: XrayInputRecord = XrayInputRecord::new();
  let search: XraySearch = search_of(&vfs, loose_id, archive_id).with_recording(&record);
  let clone: XraySearch = search.clone();

  search.find("configs\\system.ltx").expect("lookup succeeds");
  clone.find("configs\\game.ltx").expect("lookup succeeds");

  assert_eq!(record.len(), 2);
}

#[test]
fn joins_what_another_record_holds_keeping_its_own_answers() {
  let (vfs, loose_id, archive_id) = loose_over_archive("join", &["configs/system.ltx"], &["configs/game.ltx"]);
  let asker: XrayInputRecord = XrayInputRecord::new();
  let nested: XrayInputRecord = XrayInputRecord::new();
  let search: XraySearch = search_of(&vfs, loose_id, archive_id).with_recording(&asker);

  search.find("configs\\system.ltx").expect("lookup succeeds");
  search
    .clone()
    .with_recording(&nested)
    .find("configs\\game.ltx")
    .expect("lookup succeeds");

  assert_eq!(asker.len(), 1, "a nested recording files under its own record");

  search.record_inputs(&nested);
  search.record_inputs(&asker);

  assert_eq!(asker.len(), 2);
  assert_eq!(asker.get(&file(&search, "configs\\game.ltx")), found(archive_id));
}

#[test]
fn a_fork_answers_as_the_snapshot_it_was_forked_from() {
  let (vfs, loose_id, archive_id) = loose_over_archive("fork", &["configs/system.ltx"], &[]);
  let record: XrayInputRecord = XrayInputRecord::new();
  let search: XraySearch = search_of(&vfs, loose_id, archive_id).with_recording(&record);

  search.find("configs\\system.ltx").expect("lookup succeeds");
  search.list_children("configs").expect("listing succeeds");

  let fork: Arc<XrayVfs> = Arc::new(vfs.fork());

  assert_eq!(fork.get_revision(), vfs.get_revision());

  for (input, answer) in record.list() {
    assert_eq!(fork.answer_input(&input), answer);
  }
}
