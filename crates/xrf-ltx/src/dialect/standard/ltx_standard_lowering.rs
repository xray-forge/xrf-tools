use std::mem;
use std::sync::Arc;

use xrf_error::{XrfError, XrfResult};

use crate::dialect::{LtxOpenSection, LtxTextInterner};
use crate::document::{LtxCheck, LtxItem, LtxItemKind};
use crate::ltx::Ltx;

/// One document's statements turned into a resolved [`Ltx`] under standard rules, a statement at a time.
///
/// Fed from a held document when a root and its includes are resolved, or straight from the parser when one document
/// is read on its own, so that a read of a few hundred thousand records never holds every statement beside what it
/// becomes.
pub(crate) struct LtxStandardLowering<'a> {
  ltx: Ltx,
  open: LtxOpenSection,
  interner: &'a mut LtxTextInterner,
  /// Whether values go through the interner as keys do.
  ///
  /// A resolved tree repeats its values across files and sections (a vanilla tree holds ten copies of each), so sharing
  /// them pays. One document read on its own is mostly records, whose values are their own numbers: interning those
  /// keeps a table of every one only to share nothing, and took over a third of lowering a spawn's graph points.
  is_sharing_values: bool,
}

impl<'a> LtxStandardLowering<'a> {
  /// A lowering that shares keys and values through `interner`, for a document of a resolved tree.
  pub(crate) fn sharing_keys_and_values(interner: &'a mut LtxTextInterner) -> Self {
    Self {
      ltx: Ltx::new(),
      open: LtxOpenSection::default(),
      interner,
      is_sharing_values: true,
    }
  }

  /// A lowering that shares keys through `interner` and gives each value its own text, for one document read on its
  /// own.
  pub(crate) fn sharing_keys(interner: &'a mut LtxTextInterner) -> Self {
    Self {
      is_sharing_values: false,
      ..Self::sharing_keys_and_values(interner)
    }
  }

  /// Lower one statement.
  ///
  /// # Errors
  ///
  /// Returns an error when a section is declared twice, an include is named twice, or the statement is a DLTX
  /// operation, which standard mode cannot evaluate.
  pub(crate) fn push<S: AsRef<str>>(&mut self, item: &LtxItem<S>) -> XrfResult {
    if item.is_patch_operation() {
      return Err(Self::new_patch_operation_error(item));
    }

    match &item.kind {
      // Carried for formatting and for an editor, and spent by nothing here.
      LtxItemKind::Comment { .. } => {}

      LtxItemKind::Include { path, .. } => {
        let path: &str = path.as_ref();

        if self.ltx.includes(path) {
          return Err(XrfError::new_ltx_parse_error(
            item.span.get_line(),
            item.span.get_column(),
            format!("Failed to parse include statement in ltx file, including '{path}' more than once"),
          ));
        }

        self.ltx.include(String::from(path));
      }

      LtxItemKind::Section { name, parents, .. } => {
        let name: &str = name.as_ref();

        // Closed first, so a section declared twice is caught by the second header rather than missed because the
        // first one had not landed yet.
        mem::take(&mut self.open).close_into(&mut self.ltx);

        if self.ltx.has_section(name) {
          return Err(XrfError::new_ltx_parse_error(
            item.span.get_line(),
            item.span.get_column(),
            format!("Duplicate sections are not allowed, looks like '{name}' is declared twice"),
          ));
        }

        self.open = LtxOpenSection::declared(name, parents);
      }

      LtxItemKind::Key { name, value, .. } => {
        let value: &str = value.as_ref().map_or("", AsRef::as_ref);
        let value: Arc<str> = if self.is_sharing_values {
          self.interner.intern(value)
        } else {
          Arc::from(value)
        };

        self.open.insert(self.interner.intern(name.as_ref()), value);
      }
    }

    Ok(())
  }

  /// The resolved config, carrying the checks its document opted out of.
  pub(crate) fn finish(self, skipped_checks: Vec<LtxCheck>) -> Ltx {
    let Self { mut ltx, open, .. } = self;

    open.close_into(&mut ltx);
    ltx.skipped_checks = skipped_checks;

    ltx
  }

  /// The diagnostic a DLTX statement gets in standard mode.
  ///
  /// Names the flag, because a patch file read without it is a mode mistake rather than a broken config.
  fn new_patch_operation_error<S: AsRef<str>>(item: &LtxItem<S>) -> XrfError {
    let statement: String = match &item.kind {
      LtxItemKind::Section { name, operation, .. } => {
        format!("section '{}[{}]'", operation.as_prefix(), name.as_ref())
      }
      LtxItemKind::Key { name, operation, .. } => {
        format!("field '{}{}'", operation.as_prefix(), name.as_ref())
      }
      _ => String::from("statement"),
    };

    XrfError::new_ltx_parse_error(
      item.span.get_line(),
      item.span.get_column(),
      format!("Found DLTX {statement}, which needs the dltx dialect; rerun with --dltx to evaluate patch files"),
    )
  }
}
