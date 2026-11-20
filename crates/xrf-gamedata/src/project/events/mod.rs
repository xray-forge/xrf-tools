//! What a gamedata verification tells whoever is watching: the events it hands out, and the printer that says them.

mod gamedata_event;
mod gamedata_listener;
mod gamedata_run_printer;

pub use gamedata_event::GamedataEvent;
pub use gamedata_listener::GamedataListener;
pub use gamedata_run_printer::GamedataRunPrinter;
