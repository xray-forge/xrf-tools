#![doc = include_str!("../README.md")]

pub(crate) mod macros;
pub(crate) mod output_channel;
pub(crate) mod output_options;
pub(crate) mod output_record;
pub(crate) mod output_verbosity;
pub(crate) mod sequence;
pub(crate) mod sink;

pub use crate::output_channel::OutputChannel;
pub use crate::output_options::OutputOptions;
pub use crate::output_record::OutputRecord;
pub use crate::output_verbosity::OutputVerbosity;
pub use crate::sequence::{OutputSequence, OutputSlot};
pub use crate::sink::{NoopOutput, Output, RecordingOutput};
