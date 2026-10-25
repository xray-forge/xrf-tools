use crate::xrf_error::XrfError;

/// The result every XRF reader, writer and command returns; `XrfResult` alone is `Result<(), XrfError>`.
pub type XrfResult<T = ()> = Result<T, XrfError>;
