use xrf_error::{XrfError, XrfResult};

use crate::dialect::{LtxStandardDialect, LtxStandardLowering, LtxTextInterner};
use crate::document::{LtxDocument, LtxParser};
use crate::ltx::Ltx;

impl LtxStandardDialect {
  /// Lowers one document, sharing its text through an interner the caller owns.
  ///
  /// A rule, not a property of the document: the document accepts a duplicate section, a repeated include and DLTX
  /// operations, and refusing them is this dialect's answer. DLTX refuses a different set, which is why the two live
  /// beside each other as dialects rather than inside the document as one lowering.
  ///
  /// The door a whole-root resolution uses: one interner spans the root and every file it includes, so a key name
  /// written in fifty of them is stored once.
  ///
  /// # Errors
  ///
  /// Returns an error when a section is declared twice, an include is named twice, or the file carries a DLTX
  /// operation, which standard mode cannot evaluate.
  pub(crate) fn lower_with(document: &LtxDocument, interner: &mut LtxTextInterner) -> XrfResult<Ltx> {
    let mut lowering: LtxStandardLowering = LtxStandardLowering::sharing_keys_and_values(interner);

    for item in &document.items {
      lowering.push(item)?;
    }

    Ok(lowering.finish(document.skipped_checks.clone()))
  }

  /// Parses and lowers one document on its own, each statement lowered as it is read and let go.
  ///
  /// Answers exactly what lowering the parsed document would, refusals included: a statement that does not parse is
  /// reported ahead of any lowering refusal, wherever each sits in the text, as it was when the whole document was
  /// parsed first. Keys are shared, values are not (see [`LtxStandardLowering`]).
  ///
  /// # Errors
  ///
  /// Returns an error when the text does not parse, or for what [`Self::lower_with`] refuses.
  pub(crate) fn parse_and_lower(text: &str) -> XrfResult<Ltx> {
    let mut parser: LtxParser = LtxParser::new(text.chars());
    let mut keys: LtxTextInterner = LtxTextInterner::default();
    let mut lowering: LtxStandardLowering = LtxStandardLowering::sharing_keys(&mut keys);
    let mut refusal: Option<XrfError> = None;

    while let Some(item) = parser.parse_next_item()? {
      // After the first refusal the rest is only parsed, for a parse error that would outrank it.
      if refusal.is_none()
        && let Err(error) = lowering.push(&item)
      {
        refusal = Some(error);
      }
    }

    match refusal {
      Some(refusal) => Err(refusal),
      None => Ok(lowering.finish(parser.take_skipped_checks())),
    }
  }
}
