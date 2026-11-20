//! Shadow rays traced on the GPU with ray queries, shared by every compiler that traces them.

pub(crate) mod ray_gpu_alpha_face;
pub(crate) mod ray_gpu_alpha_texture;
pub(crate) mod ray_gpu_alpha_textures;
pub(crate) mod ray_gpu_batch;
pub(crate) mod ray_gpu_batch_size;
pub(crate) mod ray_gpu_origin;
pub(crate) mod ray_gpu_packet;
pub(crate) mod ray_gpu_plan;
pub(crate) mod ray_gpu_single;
pub(crate) mod ray_gpu_tracer;
