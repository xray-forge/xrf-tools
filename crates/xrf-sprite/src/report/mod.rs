//! What sprite runs tell a person: the notices they hand their listener and the printer that renders them with the
//! counts.

mod sprite_listener;
mod sprite_notice;
#[cfg(test)]
mod sprite_notice_recorder;
mod sprite_run_printer;

pub use sprite_listener::SpriteListener;
pub use sprite_notice::SpriteNotice;
#[cfg(test)]
pub(crate) use sprite_notice_recorder::SpriteNoticeRecorder;
pub use sprite_run_printer::SpriteRunPrinter;
