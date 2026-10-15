use crate::PipelineCache;

#[test]
fn hands_the_driver_only_data_whose_checksum_holds() {
  let checked: Vec<u8> = PipelineCache::to_checked(b"driver pipelines");

  assert_eq!(PipelineCache::read_checked(&checked), Some(&b"driver pipelines"[..]));

  let mut corrupted: Vec<u8> = checked.clone();

  corrupted[8] ^= 1;

  assert_eq!(PipelineCache::read_checked(&corrupted), None);
  assert_eq!(PipelineCache::read_checked(&checked[..3]), None);
}
