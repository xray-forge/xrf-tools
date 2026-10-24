//! Where the application writes: its own data, its preferences and caches, and the output tools write by default.

mod app_directories;

#[cfg(test)]
mod tests;

pub use app_directories::AppDirectories;
