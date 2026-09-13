use std::fmt::{Display, Formatter, Result as FmtResult};

use serde::{Deserialize, Serialize};

use crate::file_extension::has_extension;

/// Declares the vocabulary once, so the spelling, the serialized name and the parser cannot drift apart.
macro_rules! declare_xray_extensions {
  ($($(#[$attribute:meta])* $variant:ident => $spelling:literal,)+) => {
    /// A file extension X-Ray game data or XRF tooling actually ships, as it is spelled on disk.
    ///
    /// One variant per real spelling, so `anm` and `anm1` are two members rather than one normalized to the other:
    /// what a reader does with them is the reader's business, and two of them already disagree about it. Membership
    /// is evidence — a tree that ships the spelling, or a reader that loads it — never a guess at what might exist.
    ///
    /// The serialized name is the spelling, written per variant rather than derived: a case convention would emit
    /// `psStatic` for a file that is spelled `ps_static` on disk, and a consumer comparing against a name table would
    /// then match nothing.
    #[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
    pub enum XrayExtension {
      $(
        $(#[$attribute])*
        #[serde(rename = $spelling)]
        $variant,
      )+
    }

    impl XrayExtension {
      /// Every declared extension, in declaration order.
      ///
      /// For a consumer enumerating the vocabulary, and for the round trip this crate's tests hold it to: each
      /// variant's spelling has to parse back to it and serialize as itself.
      pub const ALL: &'static [Self] = &[$(Self::$variant,)+];

      /// The longest declared spelling, which is what bounds the buffer [`Self::parse`] folds into.
      ///
      /// Computed from the table rather than written down, so adding a longer spelling cannot silently make it
      /// unparseable.
      const LONGEST_SPELLING: usize = {
        let spellings: &[&str] = &[$($spelling,)+];
        let mut longest: usize = 0;
        let mut index: usize = 0;

        while index < spellings.len() {
          if spellings[index].len() > longest {
            longest = spellings[index].len();
          }

          index += 1;
        }

        longest
      };

      /// The spelling this extension is written with on disk, which is also the name it serializes as.
      pub const fn as_str(self) -> &'static str {
        match self {
          $(Self::$variant => $spelling,)+
        }
      }

      /// The extension `value` spells, whatever case it was authored in, or `None` for one nothing here models.
      ///
      /// `value` is the undotted token [`crate::get_file_extension`] answers with, not a file name.
      ///
      /// Folds into a stack buffer rather than an allocation, and compares in one match rather than walking the
      /// table: this runs once per entry of a name table holding hundreds of thousands of them.
      pub fn parse(value: &str) -> Option<Self> {
        let bytes: &[u8] = value.as_bytes();

        if bytes.len() > Self::LONGEST_SPELLING {
          return None;
        }

        let mut folded: [u8; Self::LONGEST_SPELLING] = [0; Self::LONGEST_SPELLING];

        folded[..bytes.len()].copy_from_slice(bytes);
        folded[..bytes.len()].make_ascii_lowercase();

        // ASCII folding rewrites only `A`-`Z`, so this re-borrows bytes that were already valid text.
        match std::str::from_utf8(&folded[..bytes.len()]).ok()? {
          $($spelling => Some(Self::$variant),)+
          _ => None,
        }
      }
    }
  };
}

declare_xray_extensions! {
  Ai => "ai",
  Anm => "anm",
  Anm1 => "anm1",
  Bat => "bat",
  Bmp => "bmp",
  CForm => "cform",
  Cmd => "cmd",
  Cs => "cs",
  Dds => "dds",
  Details => "details",
  Dm => "dm",
  Ds => "ds",
  Efd => "efd",
  EnvMod => "env_mod",
  FogVol => "fog_vol",
  Game => "game",
  Geom => "geom",
  GeomX => "geomx",
  Gs => "gs",
  H => "h",
  /// The D3D11 shader sources IX-Ray ships loose in `shaders\d3d11\`, beside the older renderers' `.ps`/`.vs` pairs.
  Hlsl => "hlsl",
  Hom => "hom",
  Htm => "htm",
  Html => "html",
  Hs => "hs",
  Ini => "ini",
  /// Both spellings of one format, because both reach a reader: `image::open` picks its decoder off the path.
  Jpeg => "jpeg",
  Jpg => "jpg",
  Json => "json",
  Lights => "lights",
  Log => "log",
  Ltx => "ltx",
  Lua => "lua",
  Md => "md",
  Ogf => "ogf",
  Ogg => "ogg",
  Ogm => "ogm",
  Omf => "omf",
  Png => "png",
  Ppe => "ppe",
  Ps => "ps",
  PsStatic => "ps_static",
  Py => "py",
  S => "s",
  /// The trailing underscore is the spelling, not a typo the vocabulary should tidy: `s_` sits beside `s` in a
  /// shaders tree and the engine's own readers treat it as another shader source.
  S_ => "s_",
  Script => "script",
  Seq => "seq",
  /// Named the way `s_` is, beside `seq` in the same trees.
  Seq_ => "seq_",
  SndStatic => "snd_static",
  Spawn => "spawn",
  Tga => "tga",
  Thm => "thm",
  /// XRF's own source spelling for a generated config, which `xrf-ltx` looks for beside an absent `.ltx` to tell a
  /// config that has not been built yet from one that is missing.
  Ts => "ts",
  Vs => "vs",
  Wallmarks => "wallmarks",
  Xml => "xml",
  /// The shader library container, `shaders.xr`. Nothing else in a tree carries it.
  Xr => "xr",
}

impl XrayExtension {
  /// Whether `name` carries this extension, compared the way [`has_extension`] compares one.
  pub fn matches(self, name: &str) -> bool {
    has_extension(name, self.as_str())
  }
}

impl Display for XrayExtension {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
    formatter.write_str(self.as_str())
  }
}

#[cfg(test)]
mod tests {
  use std::collections::HashSet;

  use super::XrayExtension;

  #[test]
  fn reads_an_extension_off_its_spelling() {
    assert_eq!(XrayExtension::parse("ltx"), Some(XrayExtension::Ltx));
    assert_eq!(XrayExtension::parse("ps_static"), Some(XrayExtension::PsStatic));
    assert_eq!(XrayExtension::parse("snd_static"), Some(XrayExtension::SndStatic));
    assert_eq!(XrayExtension::parse("geomx"), Some(XrayExtension::GeomX));
    assert_eq!(XrayExtension::parse("cform"), Some(XrayExtension::CForm));
  }

  #[test]
  fn tells_apart_the_spellings_a_normalizing_vocabulary_would_merge() {
    // Each pair is two files a tree really holds, and a reader is allowed to treat them differently.
    assert_ne!(XrayExtension::parse("anm"), XrayExtension::parse("anm1"));
    assert_ne!(XrayExtension::parse("s"), XrayExtension::parse("s_"));
    assert_ne!(XrayExtension::parse("seq"), XrayExtension::parse("seq_"));
    assert_ne!(XrayExtension::parse("htm"), XrayExtension::parse("html"));
  }

  #[test]
  fn reads_a_spelling_whatever_case_it_was_authored_in() {
    assert_eq!(XrayExtension::parse("DDS"), Some(XrayExtension::Dds));
    assert_eq!(XrayExtension::parse("Ltx"), Some(XrayExtension::Ltx));
    assert_eq!(XrayExtension::parse("PS_STATIC"), Some(XrayExtension::PsStatic));
  }

  #[test]
  fn reads_nothing_off_a_spelling_the_vocabulary_does_not_hold() {
    assert_eq!(XrayExtension::parse("psd"), None);
    assert_eq!(XrayExtension::parse(""), None);
    // Longer than any spelling, which is the early exit rather than a match arm.
    assert_eq!(XrayExtension::parse("ltx_but_much_longer"), None);
  }

  #[test]
  fn reads_nothing_off_a_dotted_spelling() {
    // `parse` takes the token a splitter answered with, never a name or a suffix.
    assert_eq!(XrayExtension::parse(".ltx"), None);
    assert_eq!(XrayExtension::parse("system.ltx"), None);
  }

  #[test]
  fn survives_a_spelling_that_is_not_ascii() {
    // Folding must not slice a multi-byte character in half, and a name table holds whatever bytes it was given.
    assert_eq!(XrayExtension::parse("ltx\u{00e9}"), None);
    assert_eq!(XrayExtension::parse("\u{0424}"), None);
  }

  #[test]
  fn every_declared_spelling_parses_back_to_the_variant_that_declared_it() {
    for extension in XrayExtension::ALL {
      assert_eq!(
        XrayExtension::parse(extension.as_str()),
        Some(*extension),
        "{extension} does not parse back to itself"
      );
      assert_eq!(
        XrayExtension::parse(&extension.as_str().to_uppercase()),
        Some(*extension),
        "{extension} does not parse back to itself in upper case"
      );
    }
  }

  #[test]
  fn no_two_variants_claim_one_spelling() {
    let mut seen: HashSet<&str> = HashSet::new();

    for extension in XrayExtension::ALL {
      assert!(seen.insert(extension.as_str()), "{extension} is declared twice");
    }

    assert_eq!(seen.len(), XrayExtension::ALL.len());
  }

  #[test]
  fn every_spelling_is_lower_case_and_undotted() {
    for extension in XrayExtension::ALL {
      let spelling: &str = extension.as_str();

      assert!(!spelling.is_empty(), "a spelling cannot be empty");
      assert!(
        spelling.chars().all(|character| !character.is_ascii_uppercase()),
        "{spelling} has to be declared folded, because that is what `parse` compares against"
      );
      assert!(!spelling.contains('.'), "{spelling} is an extension, not a suffix");
    }
  }

  #[test]
  fn serializes_as_the_spelling_on_disk_rather_than_as_the_variant() {
    // The reason every rename is written out: a case convention would emit `psStatic` here.
    assert_eq!(
      serde_json::to_string(&XrayExtension::PsStatic).expect("serializes"),
      "\"ps_static\""
    );
    assert_eq!(
      serde_json::from_str::<XrayExtension>("\"snd_static\"").expect("deserializes"),
      XrayExtension::SndStatic
    );

    for extension in XrayExtension::ALL {
      assert_eq!(
        serde_json::to_string(extension).expect("serializes"),
        format!("\"{}\"", extension.as_str())
      );
    }
  }

  #[test]
  fn answers_whether_a_name_carries_this_extension() {
    assert!(XrayExtension::Ltx.matches("configs\\system.ltx"));
    assert!(XrayExtension::Ltx.matches("configs\\SYSTEM.LTX"));
    assert!(XrayExtension::S.matches("shaders\\r1\\.s"));

    assert!(!XrayExtension::Ltx.matches("configs\\system.xml"));
    // The splitter's rule, reached through the variant: a name merely ending in the spelling is not that extension.
    assert!(!XrayExtension::Xml.matches("notes.myxml"));
    assert!(!XrayExtension::Ltx.matches("system"));
  }

  #[test]
  fn matching_a_name_agrees_with_reading_its_extension() {
    // Two doors onto one comparison, which is the whole point of the crate; a test says so rather than a comment.
    for name in ["configs\\system.ltx", "TEXTURES\\A.DDS", ".s", "notes.myxml", "level"] {
      for extension in XrayExtension::ALL {
        assert_eq!(
          extension.matches(name),
          crate::XrayExtensionOf::of(name).known() == Some(*extension),
          "{extension} disagrees with itself about '{name}'"
        );
      }
    }
  }

  #[test]
  fn renders_as_the_spelling_it_is_written_with() {
    assert_eq!(XrayExtension::EnvMod.to_string(), "env_mod");
    assert_eq!(format!("{}", XrayExtension::Xr), "xr");
  }
}
