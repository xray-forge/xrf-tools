/// Declares a stable, non-empty report identifier: a string newtype that serializes as the plain string.
macro_rules! identifier {
  ($name:ident, $kind:literal) => {
    #[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
    #[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, ::serde::Serialize)]
    #[serde(transparent)]
    pub struct $name(String);

    impl $name {
      /// Creates a stable identifier.
      ///
      /// # Errors
      ///
      /// Returns [`IdentifierError`](crate::IdentifierError) when `value` is empty.
      pub fn new(value: impl Into<String>) -> Result<Self, $crate::IdentifierError> {
        let value: String = value.into();

        if value.is_empty() {
          Err($crate::IdentifierError::empty($kind))
        } else {
          Ok(Self(value))
        }
      }

      /// Creates a stable identifier a check or rule names itself by, spelled in its source.
      ///
      /// # Panics
      ///
      /// Panics when `value` is empty, which only a source spelling it so can make it.
      pub fn of_static(value: &'static str) -> Self {
        Self::new(value).expect("a check or rule names itself by a non-empty literal")
      }

      pub fn as_str(&self) -> &str {
        &self.0
      }
    }

    impl ::std::fmt::Display for $name {
      fn fmt(&self, formatter: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        formatter.write_str(&self.0)
      }
    }
  };
}

pub(crate) use identifier;
