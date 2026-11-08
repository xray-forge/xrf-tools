#![doc = include_str!("../README.md")]

/// Starts the Tracy client, before any thread opens a zone; call once, first thing in a binary's `main`.
///
/// A zone opened before it panics in a `tracy` build, as the client it streams to is not running.
pub fn start() {
  #[cfg(feature = "tracy")]
  tracy_client::Client::start();
}

/// Puts a message on the capture's timeline, as a driver marks where each measured scene begins.
pub fn send_message(text: &str) {
  #[cfg(feature = "tracy")]
  if let Some(client) = tracy_client::Client::running() {
    client.message(text, 0);
  }

  #[cfg(not(feature = "tracy"))]
  let _ = text;
}

/// How many frames of a call stack each allocation records: enough to name the owner past the collection types and the
/// allocator itself, few enough that a capture of a level open stays usable.
#[cfg(feature = "tracy-memory")]
const ALLOCATION_STACK_DEPTH: u16 = 12;

#[cfg(feature = "tracy-memory")]
#[global_allocator]
static ALLOCATOR: tracy_client::ProfiledAllocator<std::alloc::System> =
  tracy_client::ProfiledAllocator::new(std::alloc::System, ALLOCATION_STACK_DEPTH);
