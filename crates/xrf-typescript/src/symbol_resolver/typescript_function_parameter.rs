use crate::TypeScriptTypeRenderer;
use crate::swc_common::{SourceMap, SourceMapper, Span};
use crate::swc_ecma_ast::{Pat, TsTypeAnn};

/// One parameter in a resolved TypeScript function signature.
#[derive(Clone, Debug, PartialEq)]
pub struct TypeScriptFunctionParameter {
  pub name: String,
  pub optional: bool,
  pub type_name: String,
}

impl TypeScriptFunctionParameter {
  /// What a parameter is named where its pattern cannot be read back.
  pub const PATTERN_NAME: &'static str = "[parameter]";

  /// Read a parameter whose contract a signature can state: an identifier, an array pattern named as written, a rest
  /// identifier, or either of the first two with a default, which makes it optional.
  ///
  /// A rest parameter's type is its annotation, the array it collects. Any other pattern answers `None`.
  pub fn from_pattern(pattern: &Pat, source_map: &SourceMap) -> Option<Self> {
    let (name, annotation, optional): (String, Option<&TsTypeAnn>, bool) = match pattern {
      Pat::Ident(binding) => (
        binding.id.sym.to_string(),
        binding.type_ann.as_deref(),
        binding.id.optional,
      ),
      Pat::Array(array) => (
        Self::read_pattern_name(array.span, source_map),
        array.type_ann.as_deref(),
        array.optional,
      ),
      // SWC holds a rest parameter's annotation on the rest pattern rather than on the identifier it binds.
      Pat::Rest(rest) => {
        let Pat::Ident(binding) = rest.arg.as_ref() else {
          return None;
        };

        (
          binding.id.sym.to_string(),
          rest.type_ann.as_deref().or(binding.type_ann.as_deref()),
          false,
        )
      }
      Pat::Assign(assign) => match assign.left.as_ref() {
        Pat::Ident(binding) => (binding.id.sym.to_string(), binding.type_ann.as_deref(), true),
        Pat::Array(array) => (
          Self::read_pattern_name(array.span, source_map),
          array.type_ann.as_deref(),
          true,
        ),
        _ => return None,
      },
      _ => return None,
    };

    Some(Self {
      name,
      optional,
      type_name: TypeScriptTypeRenderer::render_annotation(annotation, source_map),
    })
  }

  /// A parameter whose pattern no contract can be read from: `[parameter]` of type `unknown`.
  pub fn new_unreadable() -> Self {
    Self {
      name: String::from(Self::PATTERN_NAME),
      optional: false,
      type_name: String::from(TypeScriptTypeRenderer::UNKNOWN),
    }
  }

  /// The parameter as a signature writes it: `name?: Type`.
  pub fn contract(&self) -> String {
    format!(
      "{}{}: {}",
      self.name,
      if self.optional { "?" } else { "" },
      self.type_name
    )
  }

  /// A destructured parameter's pattern as written, up to its type annotation.
  fn read_pattern_name(span: Span, source_map: &SourceMap) -> String {
    let Ok(snippet) = source_map.span_to_snippet(span) else {
      return String::from(Self::PATTERN_NAME);
    };
    let mut nesting: usize = 0;

    for (index, character) in snippet.char_indices() {
      match character {
        '[' | '(' | '{' => nesting += 1,
        ']' | ')' | '}' => nesting = nesting.saturating_sub(1),
        ':' if nesting == 0 => return snippet[..index].trim_end().into(),
        _ => {}
      }
    }

    snippet.trim_end().into()
  }
}
