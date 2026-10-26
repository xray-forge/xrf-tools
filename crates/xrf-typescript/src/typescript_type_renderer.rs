use swc_common::{SourceMap, SourceMapper, Spanned};
use swc_ecma_ast::{
  TsEntityName, TsFnOrConstructorType, TsFnParam, TsFnType, TsImportType, TsKeywordTypeKind, TsLit, TsType, TsTypeAnn,
  TsTypeOperatorOp, TsTypeParamInstantiation, TsTypeQueryExpr, TsUnionOrIntersectionType,
};

/// Renders TypeScript type nodes as stable TypeScript-like text.
///
/// A supported node renders from the tree rather than the source, so two spellings of one type read the same: `T[]`
/// and `Array<T>` both as `Array<T>`, with every union and intersection parenthesized.
pub struct TypeScriptTypeRenderer;

impl TypeScriptTypeRenderer {
  /// What a type renders as where none is written, or where its source cannot be read back.
  pub const UNKNOWN: &'static str = "unknown";

  /// What a node outside the supported syntax renders as.
  pub const UNSUPPORTED: &'static str = "unsupported";

  /// Render a type from its tree alone, writing each unsupported node as `unsupported`.
  ///
  /// A type holding an unsupported node is logged as a warning.
  pub fn render(ts_type: &TsType) -> String {
    let mut is_supported: bool = true;
    let text: String = Self::write(ts_type, &mut is_supported);

    if !is_supported {
      log::warn!("Rendered a TypeScript type holding unsupported syntax as: {text}");
    }

    text
  }

  /// Render a type canonically, or as its source text, whitespace collapsed, where the tree holds unsupported syntax.
  ///
  /// The source keeps what the tree rendering would lose, such as a type literal or a mapped type, whole. A type whose
  /// source the map cannot give back renders as `unknown`.
  pub fn render_canonical(ts_type: &TsType, source_map: &SourceMap) -> String {
    let mut is_supported: bool = true;
    let text: String = Self::write(ts_type, &mut is_supported);

    if is_supported {
      return text;
    }

    match source_map.span_to_snippet(ts_type.span()) {
      Ok(snippet) => snippet.split_whitespace().collect::<Vec<&str>>().join(" "),
      Err(error) => {
        log::warn!("Failed to read the source of a TypeScript type: {error:?}");

        String::from(Self::UNKNOWN)
      }
    }
  }

  /// Render an optional annotation canonically, `unknown` where none is written.
  pub fn render_annotation(annotation: Option<&TsTypeAnn>, source_map: &SourceMap) -> String {
    annotation.map_or_else(
      || String::from(Self::UNKNOWN),
      |annotation| Self::render_canonical(&annotation.type_ann, source_map),
    )
  }

  /// Write one node, clearing `is_supported` where it or a node inside it renders as `unsupported`.
  fn write(ts_type: &TsType, is_supported: &mut bool) -> String {
    match ts_type {
      TsType::TsKeywordType(keyword_type) => String::from(Self::write_keyword(keyword_type.kind)),
      TsType::TsTypeRef(type_ref) => {
        let name: String = Self::write_entity_name(&type_ref.type_name);

        match &type_ref.type_params {
          Some(type_params) => format!("{name}<{}>", Self::write_type_arguments(type_params, is_supported)),
          None => name,
        }
      }
      TsType::TsArrayType(array_type) => format!("Array<{}>", Self::write(&array_type.elem_type, is_supported)),
      TsType::TsUnionOrIntersectionType(TsUnionOrIntersectionType::TsUnionType(union)) => {
        format!("({})", Self::write_joined(&union.types, " | ", is_supported))
      }
      TsType::TsUnionOrIntersectionType(TsUnionOrIntersectionType::TsIntersectionType(intersection)) => {
        format!("({})", Self::write_joined(&intersection.types, " & ", is_supported))
      }
      TsType::TsLitType(literal_type) => Self::write_literal(&literal_type.lit, is_supported),
      TsType::TsTypeOperator(type_operator) => {
        let operator: &str = match type_operator.op {
          TsTypeOperatorOp::KeyOf => "keyof",
          TsTypeOperatorOp::Unique => "unique",
          TsTypeOperatorOp::ReadOnly => "readonly",
        };

        format!("{operator} {}", Self::write(&type_operator.type_ann, is_supported))
      }
      TsType::TsTypeQuery(type_query) => {
        let name: String = match &type_query.expr_name {
          TsTypeQueryExpr::TsEntityName(entity_name) => Self::write_entity_name(entity_name),
          TsTypeQueryExpr::Import(import_type) => Self::write_import(import_type, is_supported),
        };

        match &type_query.type_args {
          Some(type_args) => format!("typeof {name}<{}>", Self::write_type_arguments(type_args, is_supported)),
          None => format!("typeof {name}"),
        }
      }
      TsType::TsFnOrConstructorType(TsFnOrConstructorType::TsFnType(function_type)) => {
        Self::write_function(function_type, is_supported)
      }
      TsType::TsTupleType(tuple_type) => {
        let elements: Vec<String> = tuple_type
          .elem_types
          .iter()
          .map(|element| Self::write(&element.ty, is_supported))
          .collect();

        format!("[{}]", elements.join(", "))
      }
      TsType::TsOptionalType(optional_type) => format!("{}?", Self::write(&optional_type.type_ann, is_supported)),
      TsType::TsRestType(rest_type) => format!("...{}", Self::write(&rest_type.type_ann, is_supported)),
      TsType::TsParenthesizedType(parenthesized_type) => {
        format!("({})", Self::write(&parenthesized_type.type_ann, is_supported))
      }
      TsType::TsImportType(import_type) => Self::write_import(import_type, is_supported),
      _ => Self::write_unsupported(is_supported),
    }
  }

  fn write_unsupported(is_supported: &mut bool) -> String {
    *is_supported = false;

    String::from(Self::UNSUPPORTED)
  }

  fn write_joined(types: &[Box<TsType>], separator: &str, is_supported: &mut bool) -> String {
    types
      .iter()
      .map(|ts_type| Self::write(ts_type, is_supported))
      .collect::<Vec<String>>()
      .join(separator)
  }

  fn write_type_arguments(type_params: &TsTypeParamInstantiation, is_supported: &mut bool) -> String {
    Self::write_joined(&type_params.params, ", ", is_supported)
  }

  fn write_function(function_type: &TsFnType, is_supported: &mut bool) -> String {
    // A generic function type's parameters are written in terms of its type parameters, which the text would drop.
    if function_type.type_params.is_some() {
      return Self::write_unsupported(is_supported);
    }

    let params: Vec<String> = function_type
      .params
      .iter()
      .map(|parameter| match parameter {
        TsFnParam::Ident(binding) => {
          let optional: &str = if binding.id.optional { "?" } else { "" };
          let type_name: String = match &binding.type_ann {
            Some(annotation) => Self::write(&annotation.type_ann, is_supported),
            None => Self::write_unsupported(is_supported),
          };

          format!("{}{optional}: {type_name}", binding.id.sym)
        }
        _ => Self::write_unsupported(is_supported),
      })
      .collect();

    format!(
      "({}) => {}",
      params.join(", "),
      Self::write(&function_type.type_ann.type_ann, is_supported)
    )
  }

  fn write_import(import_type: &TsImportType, is_supported: &mut bool) -> String {
    let qualifier: String = import_type
      .qualifier
      .as_ref()
      .map(|qualifier| format!(".{}", Self::write_entity_name(qualifier)))
      .unwrap_or_default();
    let arguments: String = import_type
      .type_args
      .as_ref()
      .map(|type_args| format!("<{}>", Self::write_type_arguments(type_args, is_supported)))
      .unwrap_or_default();

    format!(
      "import({}){qualifier}{arguments}",
      Self::quote(&import_type.arg.value.to_string_lossy())
    )
  }

  fn write_entity_name(name: &TsEntityName) -> String {
    match name {
      TsEntityName::Ident(identifier) => identifier.sym.to_string(),
      TsEntityName::TsQualifiedName(qualified) => {
        format!("{}.{}", Self::write_entity_name(&qualified.left), qualified.right.sym)
      }
    }
  }

  fn write_literal(literal: &TsLit, is_supported: &mut bool) -> String {
    match literal {
      TsLit::Str(value) => Self::quote(&value.value.to_string_lossy()),
      TsLit::Bool(value) => value.value.to_string(),
      TsLit::Number(value) => value.value.to_string(),
      TsLit::BigInt(value) => format!("{}n", value.value),
      TsLit::Tpl(_) => Self::write_unsupported(is_supported),
    }
  }

  /// A string as a double-quoted TypeScript literal, whichever quotes the source used.
  fn quote(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
  }

  fn write_keyword(kind: TsKeywordTypeKind) -> &'static str {
    match kind {
      TsKeywordTypeKind::TsAnyKeyword => "any",
      TsKeywordTypeKind::TsUnknownKeyword => "unknown",
      TsKeywordTypeKind::TsStringKeyword => "string",
      TsKeywordTypeKind::TsNumberKeyword => "number",
      TsKeywordTypeKind::TsBooleanKeyword => "boolean",
      TsKeywordTypeKind::TsBigIntKeyword => "bigint",
      TsKeywordTypeKind::TsObjectKeyword => "object",
      TsKeywordTypeKind::TsSymbolKeyword => "symbol",
      TsKeywordTypeKind::TsVoidKeyword => "void",
      TsKeywordTypeKind::TsUndefinedKeyword => "undefined",
      TsKeywordTypeKind::TsNullKeyword => "null",
      TsKeywordTypeKind::TsNeverKeyword => "never",
      TsKeywordTypeKind::TsIntrinsicKeyword => "intrinsic",
    }
  }
}

#[cfg(test)]
mod tests {
  use std::fs;
  use std::path::PathBuf;

  use swc_ecma_ast::{Decl, ModuleDecl, ModuleItem, Program, TsType};
  use xrf_test_utils::GeneratedResources;

  use super::TypeScriptTypeRenderer;
  use crate::TypeScriptSource;

  /// Each exported type alias of `source`, rendered from its tree and canonically.
  fn render(name: &str, source: &str) -> Vec<(String, String)> {
    let path: PathBuf = GeneratedResources::create_dir(&format!("typescript-type-renderer/{name}"))
      .unwrap()
      .join("types.ts");

    fs::write(&path, source).unwrap();

    let source: TypeScriptSource = TypeScriptSource::parse_file(&path).unwrap();
    let Program::Module(module) = &source.program else {
      panic!("Expected a module");
    };

    module
      .body
      .iter()
      .filter_map(|item| match item {
        ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(export)) => match &export.decl {
          Decl::TsTypeAlias(alias) => Some(alias.type_ann.as_ref()),
          _ => None,
        },
        _ => None,
      })
      .map(|ts_type: &TsType| {
        (
          TypeScriptTypeRenderer::render(ts_type),
          TypeScriptTypeRenderer::render_canonical(ts_type, &source.source_map),
        )
      })
      .collect()
  }

  #[test]
  fn renders_one_spelling_for_equivalent_types() {
    let rendered = render(
      "equivalent",
      "export type A = string[];\nexport type B = Array<string>;\nexport type C = import('./x').Y<number>;",
    );

    assert_eq!(rendered[0].1, "Array<string>");
    assert_eq!(rendered[1].1, "Array<string>");
    assert_eq!(rendered[2].1, "import(\"./x\").Y<number>");
  }

  #[test]
  fn falls_back_to_the_source_for_unsupported_syntax_anywhere_in_a_type() {
    let rendered = render(
      "unsupported",
      "export type A = { id:   string } | null;\nexport type B = <T>(value: T) => T;\nexport type C = `a${string}`;",
    );

    assert_eq!(
      rendered[0],
      (
        String::from("(unsupported | null)"),
        String::from("{ id: string } | null")
      )
    );
    assert_eq!(rendered[1].1, "<T>(value: T) => T");
    assert_eq!(rendered[2].1, "`a${string}`");
  }

  #[test]
  fn keeps_a_type_named_like_the_placeholder_canonical() {
    let rendered = render(
      "placeholder-name",
      "export type A = unsupportedHandler[];\nexport type B = \"unsupported\";",
    );

    assert_eq!(rendered[0].1, "Array<unsupportedHandler>");
    assert_eq!(rendered[1].1, "\"unsupported\"");
  }

  #[test]
  fn escapes_quotes_in_string_literal_types() {
    let rendered = render("quotes", "export type A = 'say \"hi\"';");

    assert_eq!(rendered[0].1, "\"say \\\"hi\\\"\"");
  }
}
