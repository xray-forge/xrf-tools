use wgpu::InstanceFlags;

use crate::context::gpu_context::GpuContext;

/// Each wgpu switch, spelled apart from the table it is read through, and the flag it turns.
const SWITCHES: [(&str, InstanceFlags); 7] = [
  ("WGPU_VALIDATION", InstanceFlags::VALIDATION),
  ("WGPU_DEBUG", InstanceFlags::DEBUG),
  ("WGPU_DISCARD_HAL_LABELS", InstanceFlags::DISCARD_HAL_LABELS),
  (
    "WGPU_ALLOW_UNDERLYING_NONCOMPLIANT_ADAPTER",
    InstanceFlags::ALLOW_UNDERLYING_NONCOMPLIANT_ADAPTER,
  ),
  ("WGPU_GPU_BASED_VALIDATION", InstanceFlags::GPU_BASED_VALIDATION),
  ("WGPU_VALIDATION_INDIRECT_CALL", InstanceFlags::VALIDATION_INDIRECT_CALL),
  ("WGPU_STRICT_WEBGPU_COMPLIANCE", InstanceFlags::STRICT_WEBGPU_COMPLIANCE),
];

#[test]
fn turns_each_wgpu_switch_s_own_flag_on_and_off() {
  for (name, flag) in SWITCHES {
    let on: InstanceFlags =
      GpuContext::apply_instance_switches(InstanceFlags::empty(), |switch| (switch.name == name).then_some(true));
    let off: InstanceFlags =
      GpuContext::apply_instance_switches(InstanceFlags::all(), |switch| (switch.name == name).then_some(false));

    assert_eq!(on, flag, "{name}");
    assert_eq!(off, InstanceFlags::all().difference(flag), "{name}");
  }
}

#[test]
fn keeps_the_build_s_flags_where_no_switch_is_set() {
  let build: InstanceFlags = InstanceFlags::from_build_config();

  assert_eq!(GpuContext::apply_instance_switches(build, |_| None), build);
}
