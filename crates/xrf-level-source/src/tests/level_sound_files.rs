use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use xrf_vfs::{XrayInputAnswer, XrayRevision, XraySearch, XrayVfs};
use xrf_world::{WorldSound, WorldSoundInfo};

use crate::sound::{LevelSoundFileEntry, LevelSoundFiles};

const KEPT: WorldSoundInfo = WorldSoundInfo {
  length: Duration::from_secs(3),
  is_stereo: false,
  min_distance: 2.0,
  max_distance: 40.0,
  base_volume: 1.0,
};

// No step holds the file, so a read of it finds nothing: a sound with info came from the kept entry.
fn read_with_kept(answer: Option<XrayInputAnswer>) -> WorldSound {
  let search: XraySearch = XraySearch::new(Arc::new(XrayVfs::new()));
  let known: Mutex<HashMap<String, LevelSoundFileEntry>> = Mutex::new(HashMap::from([(
    String::from("ambient\\drip"),
    LevelSoundFileEntry {
      answer,
      info: Some(KEPT),
    },
  )]));

  LevelSoundFiles::new(&search, &known).get(" ambient\\drip ")
}

#[test]
fn takes_what_was_kept_of_a_file_its_snapshot_answers_alike() {
  let search: XraySearch = XraySearch::new(Arc::new(XrayVfs::new()));
  let answer: Option<XrayInputAnswer> = search.answer_file(&LevelSoundFiles::resolve_path("ambient\\drip"));

  assert_eq!(read_with_kept(answer).info, Some(KEPT));
}

// The map is carried into later snapshots, so a reader of an older one can file what it read there late.
#[test]
fn reads_again_a_file_kept_from_a_snapshot_answering_it_otherwise() {
  let search: XraySearch = XraySearch::new(Arc::new(XrayVfs::new()));
  let answer: XrayInputAnswer = search
    .answer_file(&LevelSoundFiles::resolve_path("ambient\\drip"))
    .expect("a logical path");
  let older: Option<XrayInputAnswer> = Some(XrayInputAnswer {
    source: answer.source,
    revision: if answer.revision == XrayRevision::MOUNTED {
      XrayRevision::MOUNTED.next()
    } else {
      XrayRevision::MOUNTED
    },
  });

  assert_ne!(
    search.answer_file(&LevelSoundFiles::resolve_path("ambient\\drip")),
    older,
    "the snapshots answer the file differently"
  );
  assert_eq!(read_with_kept(older).info, None, "read again, and not found");
}
