//! What reaches a person at the terminal: the streams and colours lines go out in, the job a long command reports
//! through, how a mount reads, and what a set of sources holds twice or cannot reach.

mod cli_job;
mod mount_description;
mod source_reachability;
mod terminal_output;
mod terminal_progress_sink;
mod terminal_stream;

pub use cli_job::CliJob;
pub use mount_description::MountDescription;
pub use source_reachability::SourceReachability;
pub use terminal_output::TerminalOutput;
pub use terminal_progress_sink::TerminalProgressSink;
