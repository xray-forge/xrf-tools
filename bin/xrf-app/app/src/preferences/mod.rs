//! What the backend remembers between runs, as opposed to what the document keeps in its own local storage.

mod preference_key;
mod preferences;

pub use preference_key::PreferenceKey;
pub use preferences::Preferences;
