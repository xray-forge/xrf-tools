use super::callable_signature::{arrow_signature, function_signature, function_type};
use super::symbol::TypeScriptSymbol;
use crate::ast::canonical_ts_type_to_string;
use crate::swc_common::SourceMap;
use crate::swc_ecma_ast::{Expr, Lit, ObjectLit, Prop, PropName};

/// Convert an object literal into named property types in declaration order.
pub fn object_symbol(object: &ObjectLit, source_map: &SourceMap) -> TypeScriptSymbol {
  let properties: Vec<(String, String)> = object
    .props
    .iter()
    .filter_map(|property| {
      let Prop::KeyValue(property) = property.as_prop()?.as_ref() else {
        return None;
      };
      let name: String = property_name(&property.key)?;

      Some((name, expression_type(property.value.as_ref(), source_map)))
    })
    .collect();

  TypeScriptSymbol::Object(properties)
}

/// Render an object-literal contract as a canonical TypeScript object type.
pub fn object_type(properties: &[(String, String)]) -> String {
  let values: String = properties
    .iter()
    .map(|(name, type_name)| format!("{name}: {type_name};"))
    .collect::<Vec<String>>()
    .join(" ");

  format!("{{ {values} }}")
}

/// Strip the wrappers that keep a value's shape: `as const`, `satisfies` and parentheses.
pub fn value_expression(expression: &Expr) -> &Expr {
  match expression {
    Expr::TsConstAssertion(assertion) => value_expression(assertion.expr.as_ref()),
    Expr::TsSatisfies(satisfies) => value_expression(satisfies.expr.as_ref()),
    Expr::Paren(parenthesized) => value_expression(parenthesized.expr.as_ref()),
    _ => expression,
  }
}

/// Derive a stable type string from a value expression.
fn expression_type(expression: &Expr, source_map: &SourceMap) -> String {
  match value_expression(expression) {
    Expr::Arrow(arrow) => function_type(&arrow_signature(arrow, source_map)),
    Expr::Fn(function) => function_type(&function_signature(&function.function, source_map)),
    Expr::Object(object) => object_symbol(object, source_map)
      .value_type()
      .expect("Object symbols have value types"),
    Expr::TsAs(assertion) => canonical_type(assertion.type_ann.as_ref(), source_map),
    Expr::Lit(Lit::Bool(_)) => String::from("boolean"),
    Expr::Lit(Lit::Num(_)) => String::from("number"),
    Expr::Lit(Lit::Str(_)) => String::from("string"),
    _ => String::from("unknown"),
  }
}

/// Return a supported property name from an object literal.
fn property_name(name: &PropName) -> Option<String> {
  match name {
    PropName::Ident(identifier) => Some(identifier.sym.to_string()),
    PropName::Str(value) => Some(value.value.to_string_lossy().to_string()),
    _ => None,
  }
}

/// Render a type annotation without failing resolution for unsupported syntax.
fn canonical_type(type_annotation: &crate::swc_ecma_ast::TsType, source_map: &SourceMap) -> String {
  canonical_ts_type_to_string(type_annotation, source_map).unwrap_or_else(|_| String::from("unknown"))
}
