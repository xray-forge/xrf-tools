use super::callable_signature::function_signature;
use super::symbol::TypeScriptSymbol;
use super::value_inference::{object_symbol, value_expression};
use crate::ast::canonical_ts_type_to_string;
use crate::swc_common::SourceMap;
use crate::swc_ecma_ast::{Decl, Expr, ModuleItem, Pat, Program, Stmt, VarDecl};

/// Return a symbol declared directly in one source module.
pub fn local_symbol(program: &Program, local_name: &str, source_map: &SourceMap) -> Option<TypeScriptSymbol> {
  let Program::Module(module) = program else {
    return None;
  };

  for item in &module.body {
    match item {
      ModuleItem::Stmt(Stmt::Decl(declaration)) => {
        if let Some(symbol) = declared_symbol(declaration, local_name, source_map) {
          return Some(symbol);
        }
      }
      ModuleItem::ModuleDecl(crate::swc_ecma_ast::ModuleDecl::ExportDecl(declaration)) => {
        if let Some(symbol) = declared_symbol(&declaration.decl, local_name, source_map) {
          return Some(symbol);
        }
      }
      _ => {}
    }
  }

  None
}

/// Resolve the symbol emitted directly by an export declaration.
pub fn exported_symbol(declaration: &Decl, export_name: &str, source_map: &SourceMap) -> Option<TypeScriptSymbol> {
  declared_symbol(declaration, export_name, source_map)
}

/// Resolve a source declaration by its local binding name.
fn declared_symbol(declaration: &Decl, local_name: &str, source_map: &SourceMap) -> Option<TypeScriptSymbol> {
  match declaration {
    Decl::Fn(function) if function.ident.sym == *local_name => Some(TypeScriptSymbol::Callable(function_signature(
      &function.function,
      source_map,
    ))),
    Decl::Var(variable) => variable_symbol(variable, local_name, source_map),
    _ => None,
  }
}

/// Extract a contract from an exported variable declaration.
fn variable_symbol(variable: &VarDecl, export_name: &str, source_map: &SourceMap) -> Option<TypeScriptSymbol> {
  variable.decls.iter().find_map(|declaration| {
    let Pat::Ident(identifier) = &declaration.name else {
      return None;
    };
    if identifier.id.sym != *export_name {
      return None;
    }

    let initializer: Option<&Expr> = declaration.init.as_deref().map(value_expression);

    match initializer {
      Some(Expr::Arrow(arrow)) => Some(TypeScriptSymbol::Callable(super::callable_signature::arrow_signature(
        arrow, source_map,
      ))),
      Some(Expr::Fn(function)) => Some(TypeScriptSymbol::Callable(function_signature(
        &function.function,
        source_map,
      ))),
      _ => identifier
        .type_ann
        .as_ref()
        .map(|annotation| {
          TypeScriptSymbol::Value(
            canonical_ts_type_to_string(annotation.type_ann.as_ref(), source_map)
              .unwrap_or_else(|_| String::from("unknown")),
          )
        })
        .or_else(|| match initializer {
          Some(Expr::Object(object)) => Some(object_symbol(object, source_map)),
          _ => None,
        }),
    }
  })
}
