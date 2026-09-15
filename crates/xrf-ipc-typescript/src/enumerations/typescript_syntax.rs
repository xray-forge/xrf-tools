//! Emitting the fragments a generated declaration is written out of, in the shape Specta writes them.

use std::borrow::Cow;

use specta::datatype::Deprecated;

/// A JSDoc block in the shape Specta writes one, so a rendered enum reads like the rest of the file.
pub(crate) fn render_docs(docs: &str, deprecated: Option<&Deprecated>, indent: &str) -> String {
  if docs.is_empty() && deprecated.is_none() {
    return String::new();
  }

  if deprecated.is_none() {
    let mut lines = docs.lines();

    if let (Some(line), None) = (lines.next(), lines.next()) {
      return format!("{indent}/** {} */\n", escape_docs(line));
    }
  }

  let mut block: String = format!("{indent}/**\n");

  for line in docs.lines() {
    block.push_str(&format!("{indent} * {}\n", escape_docs(line)));
  }

  if let Some(deprecated) = deprecated {
    match deprecated
      .note
      .as_deref()
      .map(str::trim)
      .filter(|note| !note.is_empty())
    {
      Some(note) => block.push_str(&format!("{indent} * @deprecated {note}\n")),
      None => block.push_str(&format!("{indent} * @deprecated\n")),
    }
  }

  block.push_str(&format!("{indent} */\n"));
  block
}

/// Documentation text that cannot terminate the comment carrying it.
fn escape_docs(text: &str) -> Cow<'_, str> {
  match text.contains("*/") {
    true => Cow::Owned(text.replace("*/", "*\\/")),
    false => Cow::Borrowed(text),
  }
}

/// A wire spelling as a TypeScript string literal.
pub(crate) fn render_string(value: &str) -> String {
  format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

#[cfg(test)]
mod tests {
  use super::{escape_docs, render_string};

  #[test]
  fn documentation_cannot_terminate_the_comment_carrying_it() {
    assert_eq!(
      escape_docs("ends a block with */ inside"),
      "ends a block with *\\/ inside"
    );
    assert_eq!(escape_docs("plain prose"), "plain prose");
  }

  #[test]
  fn a_spelling_is_quoted_as_the_literal_it_crosses_as() {
    assert_eq!(render_string("thm"), "\"thm\"");
    // Engine paths are spellings too, and a backslash that reached the output raw would escape what follows it.
    assert_eq!(render_string("act\\arm"), "\"act\\\\arm\"");
    assert_eq!(render_string("say \"no\""), "\"say \\\"no\\\"\"");
  }
}
