use std::time::Duration;

use xrf_math::glam::Vec3;

use crate::read::config_value::ConfigValue;

#[test]
fn reads_numbers_flags_text_and_vectors_as_x_ray_writes_them() {
  assert_eq!(f32::parse(" 1.5 "), Ok(1.5));
  assert_eq!(u32::parse("3"), Ok(3));
  assert_eq!(u32::parse("3.7"), Ok(3));
  assert_eq!(bool::parse("On"), Ok(true));
  assert_eq!(bool::parse("no"), Ok(false));
  assert_eq!(String::parse("  stalker "), Ok(String::from("stalker")));
  assert_eq!(Vec3::parse("0.30, 0.90, 0.30"), Ok(Vec3::new(0.3, 0.9, 0.3)));
  assert_eq!(
    Duration::parse("09:30:15"),
    Ok(Duration::from_secs(9 * 3_600 + 30 * 60 + 15))
  );
}

#[test]
fn says_why_a_value_is_not_of_its_kind() {
  assert!(f32::parse("fast").is_err());
  assert!(u32::parse("-2").is_err());
  assert!(bool::parse("maybe").is_err());
  assert!(String::parse("   ").is_err());
  assert!(Vec3::parse("1, 2").is_err());
  assert!(Duration::parse("25:00:00").is_err());
  assert!(Duration::parse("9:00").is_err());
}
