//! Decoding one entry of the open subject for display: a sound's or a picture's facts, its stored bytes for the
//! webview to play or draw, and a texture as a picture. Addressed by the session and the entry's name like every other
//! read of a subject, so a volume set is read out of what is open rather than mounted a second time.

pub(crate) mod commands;
pub(crate) mod routes;
