use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

use crate::assets::FileStamp;
use crate::assets::asset_change_order::AssetChangeOrder;
use crate::assets::asset_change_ticket::AssetChangeTicket;

/// A ticket dropped undelivered, as a fan-out that failed drops its own, gives its turn up: the events issued after it
/// are heard all the same, in their order.
#[test]
fn a_ticket_given_up_lets_later_changes_be_heard() {
  let order: Arc<AssetChangeOrder> = Arc::new(AssetChangeOrder::default());
  let heard: Arc<Mutex<Vec<u32>>> = Arc::new(Mutex::new(Vec::new()));
  let given_up: AssetChangeTicket = order.issue_next();
  let second: AssetChangeTicket = order.issue_next();
  let third: AssetChangeTicket = order.issue_next();

  for (ticket, number) in [(third, 3), (second, 2)] {
    let heard: Arc<Mutex<Vec<u32>>> = Arc::clone(&heard);

    ticket.deliver(move || heard.lock().unwrap().push(number));
  }

  assert!(
    heard.lock().unwrap().is_empty(),
    "nothing is heard before the first turn"
  );

  drop(given_up);

  assert_eq!(*heard.lock().unwrap(), vec![2, 3]);
}

#[test]
fn a_stamp_tells_a_missing_file_from_an_unreadable_one() {
  let root: PathBuf = build_absolute_generated_test_resource_path("core/assets/stamp");
  let path: PathBuf = root.join("ston_beton05.thm");

  let _ = std::fs::remove_dir_all(&root);

  std::fs::create_dir_all(&root).expect("case directory");

  assert_eq!(FileStamp::read(&path).expect("absent is not an error"), None);

  std::fs::write(&path, b"twelve bytes").expect("file is writable");

  assert_eq!(
    FileStamp::read(&path)
      .expect("present is readable")
      .map(|stamp| stamp.size),
    Some(12)
  );
}

#[test]
fn a_stamp_is_the_size_and_the_modification_time_together() {
  // Both, because a rewrite that happens to land on the same length is exactly the change a size alone would miss.
  let stamp: FileStamp = FileStamp {
    size: 12,
    modified_ms: 1_700_000_000_000,
  };

  assert_ne!(stamp, FileStamp { size: 13, ..stamp });
  assert_ne!(
    stamp,
    FileStamp {
      modified_ms: 1_700_000_000_001,
      ..stamp
    }
  );
}
