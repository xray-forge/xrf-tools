use std::sync::Arc;

use crate::registry::hook_kind::HookKind;
use crate::registry::hook_point::HookPoint;
use crate::registry::hook_registry::HookRegistry;

type Guard = dyn Fn(u32) -> bool + Send + Sync;

type Modifier = dyn Fn(f32) -> f32 + Send + Sync;

const CAN_ENTER: HookPoint<Guard> = HookPoint::new("test.can_enter", HookKind::Guard);

fn registry() -> HookRegistry {
  let mut hooks: HookRegistry = HookRegistry::default();

  hooks
    .register(&CAN_ENTER, Arc::new(|value: u32| value < 10) as Arc<Guard>)
    .expect("a new point");

  hooks
}

#[test]
fn calls_a_points_default_until_a_script_replaces_it() {
  let mut hooks: HookRegistry = registry();

  assert!((hooks.get(&CAN_ENTER).expect("registered"))(3));
  assert!(!(hooks.get(&CAN_ENTER).expect("registered"))(30));

  hooks
    .replace("test.can_enter", Arc::new(|_: u32| false) as Arc<Guard>)
    .expect("a point to replace");

  assert!(!(hooks.get(&CAN_ENTER).expect("registered"))(3));
  assert_eq!(hooks.get_kind("test.can_enter"), Some(HookKind::Guard));
  assert_eq!(hooks.len(), 1);
}

#[test]
fn wraps_the_entry_it_replaces() {
  let mut hooks: HookRegistry = registry();
  let wrapped: Arc<Guard> = hooks
    .replace("test.can_enter", Arc::new(|_: u32| true) as Arc<Guard>)
    .expect("a point to replace");

  hooks
    .replace(
      "test.can_enter",
      Arc::new(move |value: u32| value != 5 && wrapped(value)) as Arc<Guard>,
    )
    .expect("a point to replace");

  let guard: Arc<Guard> = hooks.get(&CAN_ENTER).expect("registered");

  assert!(guard(4));
  assert!(!guard(5));
  assert!(!guard(12));
}

#[test]
fn refuses_a_point_twice_an_unknown_one_and_a_function_of_another_type() {
  let mut hooks: HookRegistry = registry();

  assert!(
    hooks
      .register(&CAN_ENTER, Arc::new(|_: u32| true) as Arc<Guard>)
      .is_err()
  );
  assert!(
    hooks
      .replace("test.unknown", Arc::new(|_: u32| true) as Arc<Guard>)
      .is_err()
  );
  assert!(
    hooks
      .replace("test.can_enter", Arc::new(|value: f32| value) as Arc<Modifier>)
      .is_err()
  );
  assert!(hooks.find::<Modifier>("test.can_enter").is_none());
  assert!(
    hooks
      .get(&HookPoint::<Guard>::new("test.never", HookKind::Guard))
      .is_err()
  );
}
