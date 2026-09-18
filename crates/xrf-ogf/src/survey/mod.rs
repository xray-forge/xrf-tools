//! A walk over the chunk tree of a visual that records what is there without interpreting it.

pub(crate) mod ogf_chunk_entry;
pub(crate) mod ogf_chunk_survey;
pub(crate) mod ogf_chunks_processor;

pub use ogf_chunk_entry::OgfChunkEntry;
pub use ogf_chunk_survey::OgfChunkSurvey;
pub use ogf_chunks_processor::OgfChunksProcessor;
