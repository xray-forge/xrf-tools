//! Reading an LZHUF blob or stream back into the bytes it codes.

pub(crate) mod lzhuf_decoder;

pub use crate::decode::lzhuf_decoder::LzhufDecoder;
