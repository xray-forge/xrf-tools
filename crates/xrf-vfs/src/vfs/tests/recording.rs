use std::path::PathBuf;
use std::sync::Arc;

use crate::vfs::tests::fake_source::{FakeArchiveSource, directory};
use crate::{
  XrayAsset, XrayAssetType, XrayCachePolicy, XrayInput, XrayInputAnswer, XrayInputKind, XrayInputRecord,
  XrayLogicalPath, XrayLookupScope, XrayMountId, XrayProbe, XrayRevision, XraySearch, XrayVfs,
};

/// A loose tree searched before an archive, returned with both mounts' ids.
fn loose_over_archive(case: &str, loose: &[&str], archived: &[&str]) -> (XrayVfs, XrayMountId, XrayMountId) {
  let root: PathBuf = directory(&format!("recording_{case}"), loose);
  let mut vfs: XrayVfs = XrayVfs::new().with_cache_policy(XrayCachePolicy::none().with(XrayAssetType::Ltx));
  let loose_id: XrayMountId = vfs.mount_directory("", &root).expect("loose root mounts");
  let archive_id: XrayMountId = vfs
    .mount("", Box::new(FakeArchiveSource::new("archive", archived)))
    .expect("archive mounts");

  (vfs, loose_id, archive_id)
}

fn probe_of(vfs: &XrayVfs, loose_id: XrayMountId, archive_id: XrayMountId) -> XrayProbe<'_> {
  vfs
    .probe()
    .with_step("loose", XrayLookupScope::only([loose_id]))
    .with_step("archive", XrayLookupScope::only([archive_id]))
}

fn file(probe: &XrayProbe, path: &str) -> XrayInput {
  XrayInput::new(
    XraySearch::of(probe.get_steps()),
    XrayInputKind::File(XrayLogicalPath::new(path).expect("test path is valid")),
  )
}

fn listing(probe: &XrayProbe, directory: Option<&str>) -> XrayInput {
  XrayInput::new(
    XraySearch::of(probe.get_steps()),
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
  let probe: XrayProbe = probe_of(&vfs, loose_id, archive_id).recording(&record);

  probe
    .resolve(XrayAssetType::Dds, "wpn\\wpn_ak74")
    .expect("lookup succeeds");
  probe
    .resolve(XrayAssetType::Dds, "wpn\\wpn_abakan")
    .expect("lookup succeeds");

  assert_eq!(record.len(), 2);
  assert_eq!(
    record.get(&file(&probe, "textures\\wpn\\wpn_ak74.dds")),
    found(loose_id)
  );
  assert_eq!(
    record.get(&file(&probe, "textures\\wpn\\wpn_abakan.dds")),
    found(archive_id)
  );
}

#[test]
fn records_a_lookup_that_finds_nothing_and_its_fallback() {
  let (vfs, loose_id, archive_id) = loose_over_archive("missing", &["textures/ed/ed_not_existing_texture.dds"], &[]);
  let record: XrayInputRecord = XrayInputRecord::new();
  let probe: XrayProbe = probe_of(&vfs, loose_id, archive_id).recording(&record);

  probe.find("textures\\absent.dds").expect("lookup succeeds");
  probe
    .resolve_with_fallback(XrayAssetType::Dds, "wpn\\absent", "ed\\ed_not_existing_texture")
    .expect("lookup succeeds");

  // Nothing answering is an answer too: a file created there later is what changes it.
  assert_eq!(
    record.get(&file(&probe, "textures\\absent.dds")),
    Some(XrayInputAnswer::default())
  );
  assert_eq!(
    record.get(&file(&probe, "textures\\wpn\\absent.dds")),
    Some(XrayInputAnswer::default())
  );
  assert_eq!(
    record.get(&file(&probe, "textures\\ed\\ed_not_existing_texture.dds")),
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
  let probe: XrayProbe = probe_of(&vfs, loose_id, archive_id).recording(&record);

  probe.list_children("textures\\wpn").expect("listing succeeds");
  probe.list_assets_of_type(XrayAssetType::Dds);
  probe.resolve(XrayAssetType::Omf, "wpn\\wpn_*").expect("mask resolves");

  let listed: Option<XrayInputAnswer> = Some(XrayInputAnswer::default());

  assert_eq!(record.len(), 3);
  assert_eq!(record.get(&listing(&probe, Some("textures\\wpn"))), listed);
  assert_eq!(record.get(&listing(&probe, None)), listed);
  assert_eq!(record.get(&listing(&probe, Some("meshes"))), listed);
}

#[test]
fn records_a_read_served_from_the_cache() {
  let (vfs, loose_id, archive_id) = loose_over_archive("cached", &["configs/system.ltx"], &[]);
  let probe: XrayProbe = probe_of(&vfs, loose_id, archive_id);
  let asset: XrayAsset = probe
    .find("configs\\system.ltx")
    .expect("lookup succeeds")
    .get_asset()
    .cloned()
    .expect("the file is found");
  let parse = |bytes: Vec<u8>| Ok(bytes.len());

  probe.read_asset_parsed(&asset, parse).expect("first read parses");

  let record: XrayInputRecord = XrayInputRecord::new();
  let recording: XrayProbe = probe.recording(&record);

  recording
    .read_asset_parsed(&asset, parse)
    .expect("second read is served");

  assert_eq!(vfs.get_cache().get_stats().hits, 1, "the second read is a hit");
  assert_eq!(record.get(&file(&recording, "configs\\system.ltx")), found(loose_id));
}

#[test]
fn records_each_input_once_with_its_first_answer() {
  let (vfs, loose_id, archive_id) = loose_over_archive("once", &["configs/system.ltx"], &[]);
  let record: XrayInputRecord = XrayInputRecord::new();
  let probe: XrayProbe = probe_of(&vfs, loose_id, archive_id).recording(&record);

  let asset: XrayAsset = probe
    .find("configs\\system.ltx")
    .expect("lookup succeeds")
    .get_asset()
    .cloned()
    .expect("the file is found");

  probe.read_asset_bytes(&asset).expect("the file reads");
  probe.find("Configs/System.ltx").expect("lookup succeeds");

  assert_eq!(record.len(), 1, "a path found, read and found again is one input");
}

#[test]
fn files_an_input_under_the_search_that_resolved_it() {
  let (vfs, loose_id, archive_id) = loose_over_archive("search", &["configs/system.ltx"], &["configs/system.ltx"]);
  let record: XrayInputRecord = XrayInputRecord::new();
  let forward: XrayProbe = probe_of(&vfs, loose_id, archive_id).recording(&record);
  let backward: XrayProbe = vfs
    .probe()
    .with_step("archive", XrayLookupScope::only([archive_id]))
    .with_step("loose", XrayLookupScope::only([loose_id]))
    .recording(&record);

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
  let probe: XrayProbe = vfs
    .probe()
    .with_step("loose", XrayLookupScope::only([loose_id]))
    .recording(&record)
    .with_step("archive", XrayLookupScope::only([archive_id]));

  probe.find("configs\\system.ltx").expect("lookup succeeds");

  assert_eq!(record.get(&file(&probe, "configs\\system.ltx")), found(archive_id));
}

#[test]
fn a_plain_probe_records_nothing() {
  let (vfs, loose_id, archive_id) = loose_over_archive("plain", &["configs/system.ltx"], &[]);
  let record: XrayInputRecord = XrayInputRecord::new();
  let plain: XrayProbe = probe_of(&vfs, loose_id, archive_id);
  let recording: XrayProbe = plain.recording(&record);

  plain.find("configs\\system.ltx").expect("lookup succeeds");
  plain.record_inputs(&record);

  assert!(
    plain.get_record().is_none(),
    "recording leaves the probe it started from plain"
  );
  assert!(recording.get_record().is_some());
  assert!(record.is_empty());
}

#[test]
fn joins_what_another_record_holds_keeping_its_own_answers() {
  let (vfs, loose_id, archive_id) = loose_over_archive("join", &["configs/system.ltx"], &["configs/game.ltx"]);
  let asker: XrayInputRecord = XrayInputRecord::new();
  let nested: XrayInputRecord = XrayInputRecord::new();
  let probe: XrayProbe = probe_of(&vfs, loose_id, archive_id).recording(&asker);

  probe.find("configs\\system.ltx").expect("lookup succeeds");
  probe
    .recording(&nested)
    .find("configs\\game.ltx")
    .expect("lookup succeeds");

  assert_eq!(asker.len(), 1, "a nested recording files under its own record");

  probe.record_inputs(&nested);
  probe.record_inputs(&asker);

  assert_eq!(asker.len(), 2);
  assert_eq!(asker.get(&file(&probe, "configs\\game.ltx")), found(archive_id));
}

#[test]
fn a_fork_answers_as_the_world_it_was_forked_from() {
  let (vfs, loose_id, archive_id) = loose_over_archive("fork", &["configs/system.ltx"], &[]);
  let record: XrayInputRecord = XrayInputRecord::new();
  let probe: XrayProbe = probe_of(&vfs, loose_id, archive_id).recording(&record);

  probe.find("configs\\system.ltx").expect("lookup succeeds");
  probe.list_children("configs").expect("listing succeeds");

  let fork: Arc<XrayVfs> = Arc::new(vfs.fork());

  assert_eq!(fork.get_revision(), vfs.get_revision());

  for (input, answer) in record.list() {
    assert_eq!(fork.answer_input(&input), answer);
  }
}
