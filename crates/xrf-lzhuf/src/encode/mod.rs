//! Coding bytes into an LZHUF blob: finding back-references, then writing literals and matches through the tree.

pub(crate) mod lzhuf_encoder;
pub(crate) mod match_finder;
pub(crate) mod source_match;

pub use crate::encode::lzhuf_encoder::LzhufEncoder;
pub(crate) use crate::encode::match_finder::MatchFinder;
pub(crate) use crate::encode::source_match::SourceMatch;
