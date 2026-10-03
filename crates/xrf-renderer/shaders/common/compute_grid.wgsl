// The flat index of an invocation of a dispatch laid over rows of workgroups (`ComputeGrid`): a row holds as many as
// the device allows in one dimension, so a count past it runs on into the next row.

fn compute_index(id: vec3<u32>, groups: vec3<u32>, workgroup: u32) -> u32 {
  return id.x + id.y * groups.x * workgroup;
}
