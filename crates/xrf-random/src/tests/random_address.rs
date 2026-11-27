use crate::stream::random_address::RandomAddress;

#[test]
fn names_a_domain_by_its_fnv_1a_hash() {
  // FNV-1a's offset basis for no bytes, and its published value for "a".
  assert_eq!(RandomAddress::domain(""), 0xcbf2_9ce4_8422_2325);
  assert_eq!(RandomAddress::domain("a"), 0xaf63_dc4c_8601_ec8c);

  const MOVEMENT: u64 = RandomAddress::domain("alife.movement");

  assert_ne!(MOVEMENT, RandomAddress::domain("alife.encounter"));
}
