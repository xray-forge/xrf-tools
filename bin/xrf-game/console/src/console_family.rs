use crate::console_context::ConsoleContext;

/// The kind of context a console runs against, named apart from any borrow of it: `Context<'a>` is the context borrowed
/// for `'a`, so a program hands its console the state it owns for one line at a time (`execute(&mut context, line)`)
/// rather than giving it a share of that state to keep. A command is written once, for every such borrow.
///
/// A context that owns its variables names itself: `type Context<'a> = Self`.
pub trait ConsoleFamily: 'static {
  type Context<'a>: ConsoleContext;
}
