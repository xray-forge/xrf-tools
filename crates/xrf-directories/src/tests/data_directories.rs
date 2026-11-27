use std::path::PathBuf;

use crate::data_directories::DataDirectories;

#[test]
fn keeps_its_files_beside_the_executable_first() {
  assert_eq!(
    DataDirectories::order_app_data_directories(Some(PathBuf::from("application")), Some(PathBuf::from("data"))),
    vec![PathBuf::from("application").join("appdata"), PathBuf::from("data")]
  );
}

#[test]
fn falls_back_to_its_data_directory_where_the_executable_takes_no_files() {
  assert_eq!(
    DataDirectories::order_app_data_directories(None, Some(PathBuf::from("data"))),
    vec![PathBuf::from("data")]
  );
  assert!(DataDirectories::order_app_data_directories(None, None).is_empty());
}

#[test]
fn keeps_its_logs_in_each_of_its_own_data_directories() {
  let directories: DataDirectories = DataDirectories::new(Some(PathBuf::from("data")));

  assert_eq!(
    directories.get_logs_directories(),
    directories
      .get_app_data_directories()
      .into_iter()
      .map(|directory| directory.join("logs"))
      .collect::<Vec<PathBuf>>()
  );
}

#[test]
fn names_the_same_folders_to_every_caller_asking_at_once() {
  let directories: DataDirectories = DataDirectories::new(Some(PathBuf::from("data")));
  let first: Vec<PathBuf> = directories.get_app_data_directories();
  let answers: Vec<Vec<PathBuf>> = std::thread::scope(|scope| {
    let asks: Vec<_> = (0..16)
      .map(|_| {
        scope.spawn(|| {
          (0..32)
            .map(|_| directories.get_app_data_directories())
            .collect::<Vec<_>>()
        })
      })
      .collect();

    asks
      .into_iter()
      .flat_map(|ask| ask.join().expect("an ask to answer"))
      .collect()
  });

  assert!(answers.iter().all(|answer| *answer == first));
}
