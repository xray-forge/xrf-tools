use crate::frame::frame_encoder::FrameEncoder;
use crate::frame::periodic::periodic_scheduler::PeriodicScheduler;
use crate::frame::timing::frame_phases::FramePhases;
use crate::shader::shader_library::ShaderLibrary;
use crate::thread::gpu_state::GpuState;

/// What readying the frame's views works with: the GPU and the shader library, the frame's encoder, the periodic
/// work's scheduler, and the phases the time loading and preparing takes is added to.
pub(crate) struct ViewWork<'w> {
  pub gpu: &'w mut GpuState,
  pub shaders: &'w ShaderLibrary,
  pub encoder: &'w mut FrameEncoder,
  pub periodic: &'w mut PeriodicScheduler,
  pub phases: &'w mut FramePhases,
}
