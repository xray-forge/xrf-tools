use std::fmt::Debug;

use xrf_engine_target::XrayEngine;

use crate::key::environment_key_use::EnvironmentKeyUse;
use crate::key::environment_value_kind::EnvironmentValueKind;

/// The keys one kind of environment section is read with: each one's spelling, how the engine parses it, and per
/// engine whether it is read and what it defaults to.
pub trait EnvironmentKey: Copy + Ord + Debug + 'static {
  /// Every key, in the order the engine reads them.
  const ALL: &'static [Self];

  /// The key as a config spells it.
  fn get_name(self) -> &'static str;

  /// How the engine parses the value.
  fn get_kind(self) -> EnvironmentValueKind;

  /// Whether the engine reads the key, and what it holds without it.
  fn get_use(self, engine: XrayEngine) -> EnvironmentKeyUse;

  /// The key a config spelling names, if it is one of these.
  fn from_name(name: &str) -> Option<Self> {
    Self::ALL.iter().copied().find(|key| key.get_name() == name)
  }
}

/// Declares one section kind's keys: the enum, its serialized spellings, and its [`EnvironmentKey`] table.
macro_rules! declare_environment_keys {
  (
    $(#[$attribute:meta])*
    pub enum $name:ident {
      $(
        $(#[$variant_attribute:meta])*
        $variant:ident = $spelling:literal: $kind:expr, vanilla: $vanilla:expr, extended: $extended:expr;
      )+
    }
  ) => {
    $(#[$attribute])*
    #[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
    pub enum $name {
      $(
        $(#[$variant_attribute])*
        #[serde(rename = $spelling)]
        $variant,
      )+
    }

    impl $crate::key::EnvironmentKey for $name {
      const ALL: &'static [Self] = &[$(Self::$variant,)+];

      fn get_name(self) -> &'static str {
        match self {
          $(Self::$variant => $spelling,)+
        }
      }

      #[allow(unused_imports)]
      fn get_kind(self) -> $crate::key::EnvironmentValueKind {
        use $crate::key::EnvironmentValueKind::*;

        match self {
          $(Self::$variant => $kind,)+
        }
      }

      #[allow(unused_imports)]
      fn get_use(self, engine: xrf_engine_target::XrayEngine) -> $crate::key::EnvironmentKeyUse {
        use $crate::key::EnvironmentDefault::*;
        use $crate::key::EnvironmentKeyUse::*;

        match (self, engine) {
          $(
            (Self::$variant, xrf_engine_target::XrayEngine::Vanilla) => $vanilla,
            (Self::$variant, xrf_engine_target::XrayEngine::Extended) => $extended,
          )+
        }
      }
    }
  };
}

pub(crate) use declare_environment_keys;
