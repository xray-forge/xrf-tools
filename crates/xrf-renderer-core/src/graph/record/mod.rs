//! What a frame graph holds of its resources and passes as they are declared.

mod graph_buffer_record;
mod graph_pass;
mod graph_pass_work;
mod graph_texture_record;

pub(crate) use graph_buffer_record::GraphBufferRecord;
pub(crate) use graph_pass::GraphPass;
pub(crate) use graph_pass_work::GraphPassWork;
pub(crate) use graph_texture_record::GraphTextureRecord;
