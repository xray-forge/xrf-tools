use std::fmt::Write;

use hyper::header::HeaderValue;

/// The secret a launch's requests carry, so nothing else on the machine can read through the transport.
#[derive(Clone, Debug)]
pub(crate) struct TransportToken {
  /// Hex, as it is sent.
  value: String,
}

impl TransportToken {
  /// Random bytes a token is made of.
  const LENGTH: usize = 32;

  const SCHEME: &'static [u8] = b"Bearer ";

  /// A token of fresh random bytes.
  pub(crate) fn generate() -> Result<Self, getrandom::Error> {
    let mut bytes: [u8; Self::LENGTH] = [0; Self::LENGTH];

    getrandom::fill(&mut bytes)?;

    Ok(Self {
      value: bytes
        .iter()
        .fold(String::with_capacity(Self::LENGTH * 2), |mut value, byte| {
          // Writing to a string cannot fail.
          let _ = write!(value, "{byte:02x}");

          value
        }),
    })
  }

  pub(crate) fn get_value(&self) -> &str {
    &self.value
  }

  /// Whether an `Authorization` header carries this token, compared in time independent of where they differ.
  pub(crate) fn is_authorizing(&self, header: Option<&HeaderValue>) -> bool {
    header
      .and_then(|value| value.as_bytes().strip_prefix(Self::SCHEME))
      .is_some_and(|presented| Self::is_same(presented, self.value.as_bytes()))
  }

  fn is_same(left: &[u8], right: &[u8]) -> bool {
    // The length is not the secret: every token is as long as every other.
    left.len() == right.len()
      && left
        .iter()
        .zip(right)
        .fold(0u8, |difference, (a, b)| difference | (a ^ b))
        == 0
  }
}
