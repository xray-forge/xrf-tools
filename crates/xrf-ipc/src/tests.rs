use serde_json::json;
use specta::Types;
use specta::datatype::{DataType, Primitive};

use crate::IpcConstant;

#[test]
fn keeps_an_f32_in_its_shortest_form() {
  assert_eq!(IpcConstant::serialize_value(&0.1_f32).unwrap().to_string(), "0.1");
  assert_eq!(IpcConstant::serialize_value(&u64::MAX).unwrap(), json!(u64::MAX));
  assert_eq!(IpcConstant::serialize_value(&f32::NAN).unwrap(), json!(null));
}

#[test]
fn reads_the_type_off_the_value() {
  assert!(matches!(
    IpcConstant::define_type(&4_u32, &mut Types::default()),
    DataType::Primitive(Primitive::u32)
  ));
}

#[test]
fn drops_the_last_segment_and_keeps_a_root() {
  assert_eq!(
    IpcConstant::enclosing_module("xrf_renderer::contract::limits"),
    "xrf_renderer::contract"
  );
  assert_eq!(IpcConstant::enclosing_module("xrf_renderer"), "xrf_renderer");
}
