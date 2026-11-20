use std::borrow::Cow;

/// One argument of a command line printed for a person to paste back into their shell.
pub(crate) struct ShellArgument;

impl ShellArgument {
  /// `text` as one argument of the platform's shell: unchanged when every character is plain, otherwise quoted.
  ///
  /// POSIX shells take it single-quoted, which leaves every character literal. Windows takes it double-quoted, which
  /// both `cmd` and PowerShell read as one argument; a `$` or backtick inside still reads as PowerShell syntax.
  pub(crate) fn quote(text: &str) -> Cow<'_, str> {
    if !text.is_empty() && text.chars().all(Self::is_plain) {
      return Cow::Borrowed(text);
    }

    if cfg!(windows) {
      Cow::Owned(format!("\"{}\"", text.replace('"', "\"\"")))
    } else {
      Cow::Owned(format!("'{}'", text.replace('\'', r"'\''")))
    }
  }

  /// Whether `character` means only itself to the shell, outside any quotes.
  fn is_plain(character: char) -> bool {
    character.is_alphanumeric()
      || matches!(character, '-' | '_' | '.' | '/' | ':' | '+' | ',')
      || (cfg!(windows) && character == '\\')
  }
}

#[cfg(test)]
mod tests {
  use super::ShellArgument;

  #[test]
  fn leaves_a_plain_path_unquoted() {
    assert_eq!(ShellArgument::quote("gamedata/configs"), "gamedata/configs");
  }

  #[test]
  fn quotes_a_path_holding_a_space_or_a_special_character() {
    for path in [
      "game data/configs",
      "gamedata&configs",
      "gamedata;configs",
      "mods(1)",
      "",
    ] {
      let quoted: String = ShellArgument::quote(path).into_owned();

      if cfg!(windows) {
        assert_eq!(quoted, format!("\"{path}\""));
      } else {
        assert_eq!(quoted, format!("'{path}'"));
      }
    }
  }

  #[test]
  #[cfg(not(windows))]
  fn escapes_a_single_quote_inside_posix_quotes() {
    assert_eq!(ShellArgument::quote("it's"), r"'it'\''s'");
  }
}
