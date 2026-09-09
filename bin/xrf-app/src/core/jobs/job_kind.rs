use serde::Serialize;

// Serialization, diagnostics and frontend constants share exactly these wire identities.
macro_rules! job_kinds {
  ($($variant:ident => $wire:literal),+ $(,)?) => {
    /// The application operations that can be registered, cancelled and rediscovered.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
    #[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
    pub enum JobKind {
      $(#[serde(rename = $wire)] $variant),+
    }
    impl JobKind {
      pub const ALL: &[Self] = &[$(Self::$variant),+];
      pub const fn as_str(self) -> &'static str {
        match self { $(Self::$variant => $wire),+ }
      }
    }
  };
}

job_kinds! {
  ArchivesExtract => "archives.extract",
  ArchivesCompare => "archives.compare",
  ArchivesPack => "archives.pack",
  ArchivesPatch => "archives.patch",
  ArchivesUnpack => "archives.unpack",
  ConfigsCheckFormat => "configs.check-format",
  ConfigsFormat => "configs.format",
  ConfigsVerify => "configs.verify",
  SpawnPack => "spawn.pack",
  SpawnUnpack => "spawn.unpack",
  SpriteEquipmentPack => "sprite-equipment.pack",
  GamedataVerify => "gamedata.verify",
  TexturesBuild => "textures.build",
  TexturesCompareEncodings => "textures.compare-encodings",
  TexturesMakeBump => "textures.make-bump",
  TexturesSave => "textures.save",
  TranslationsBuild => "translations.build",
  TranslationsCheckFormat => "translations.check-format",
  TranslationsFormat => "translations.format",
  TranslationsParse => "translations.parse",
  TranslationsVerify => "translations.verify",
}

impl std::fmt::Display for JobKind {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    formatter.write_str(self.as_str())
  }
}
