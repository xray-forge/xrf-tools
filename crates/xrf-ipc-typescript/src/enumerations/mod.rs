//! Giving every identity the frontend compares against a name the compiler resolves.
//!
//! A string union names no member, so every frontend use of one is a quoted literal that nothing checks against the
//! Rust source: a misspelling is a type error only if it is also not a member of the union, and a renamed variant
//! silently becomes a different valid string somewhere else. An enum gives each identity a name the compiler
//! resolves, which is what makes a rename fail at the call site rather than at runtime.
//!
//! Two shapes reach that, and [`EnumerationSubject`] is what tells them apart:
//!
//! - A set of identities becomes a TypeScript enum **replacing** the union Specta renders, with the derived `${E..}`
//!   union beside it. Both are emitted because they answer different questions: the enum is what a caller writes and
//!   what a command parameter demands; the union is what a value read back is written against, and what a `Record<>`
//!   key, a fixture or a mock is written against, where naming every member would be noise. Which of the two a
//!   generated position gets is decided by direction, never by the type: see [`Enumerations::references`].
//! - An internally tagged union keeps the object shapes Specta renders and gains an enum of its **discriminants**,
//!   declared before it. That enum stands in for the tag and never for the value, so it is not offered to the
//!   parameter rewrite.

mod enumeration;
mod enumeration_member;
mod enumeration_subject;
mod enumerations;
mod typescript_syntax;

pub use enumerations::Enumerations;
