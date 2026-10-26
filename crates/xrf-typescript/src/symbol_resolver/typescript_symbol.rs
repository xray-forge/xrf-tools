use super::typescript_function_signature::TypeScriptFunctionSignature;
use super::typescript_object_property::TypeScriptObjectProperty;
use crate::TypeScriptTypeRenderer;
use crate::swc_common::SourceMap;
use crate::swc_ecma_ast::{
  Decl, Expr, Lit, ModuleDecl, ModuleItem, ObjectLit, Pat, Program, Prop, PropName, Stmt, VarDecl,
};

/// The TypeScript contract resolved for one source symbol.
#[derive(Clone, Debug, PartialEq)]
pub enum TypeScriptSymbol {
  Callable(TypeScriptFunctionSignature),
  Value(String),
  Object(Vec<TypeScriptObjectProperty>),
}

impl TypeScriptSymbol {
  /// The symbol's type as TypeScript writes it: a function type, the declared type, or an object type literal.
  pub fn type_name(&self) -> String {
    match self {
      Self::Callable(signature) => signature.type_name(),
      Self::Value(type_name) => type_name.clone(),
      Self::Object(properties) => {
        let members: Vec<String> = properties
          .iter()
          .map(|property| format!("{}: {};", property.name, property.type_name))
          .collect();

        format!("{{ {} }}", members.join(" "))
      }
    }
  }

  /// The type of a named property of an object-literal symbol.
  pub fn property_type(&self, property_name: &str) -> Option<&str> {
    let Self::Object(properties) = self else {
      return None;
    };

    properties
      .iter()
      .find(|property| property.name == property_name)
      .map(|property| property.type_name.as_str())
  }

  /// The symbol a source's top-level declaration binds to `name`, exported or not.
  ///
  /// A source without imports or exports parses as a script, whose declarations are read the same way.
  pub(crate) fn find_declared(program: &Program, name: &str, source_map: &SourceMap) -> Option<Self> {
    match program {
      Program::Module(module) => module.body.iter().find_map(|item| match item {
        ModuleItem::Stmt(Stmt::Decl(declaration)) => Self::from_declaration(declaration, name, source_map),
        ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(export)) => {
          Self::from_declaration(&export.decl, name, source_map)
        }
        _ => None,
      }),
      Program::Script(script) => script.body.iter().find_map(|statement| match statement {
        Stmt::Decl(declaration) => Self::from_declaration(declaration, name, source_map),
        _ => None,
      }),
    }
  }

  /// The symbol one declaration binds to `name`, where it binds that name with a contract that can be read.
  pub(crate) fn from_declaration(declaration: &Decl, name: &str, source_map: &SourceMap) -> Option<Self> {
    match declaration {
      Decl::Fn(function) if function.ident.sym == *name => Some(Self::Callable(
        TypeScriptFunctionSignature::from_function(&function.function, source_map),
      )),
      Decl::Var(variable) => Self::from_variable(variable, name, source_map),
      _ => None,
    }
  }

  /// A variable's contract: a function initializer's signature, else its annotation, else the type its initializer
  /// asserts, else its object literal's properties.
  fn from_variable(variable: &VarDecl, name: &str, source_map: &SourceMap) -> Option<Self> {
    let (identifier, initializer) = variable.decls.iter().find_map(|declarator| match &declarator.name {
      Pat::Ident(identifier) if identifier.id.sym == *name => Some((identifier, declarator.init.as_deref())),
      _ => None,
    })?;

    match (initializer.map(Self::unwrap_value), identifier.type_ann.as_deref()) {
      (Some(Expr::Arrow(arrow)), _) => Some(Self::Callable(TypeScriptFunctionSignature::from_arrow(
        arrow, source_map,
      ))),
      (Some(Expr::Fn(function)), _) => Some(Self::Callable(TypeScriptFunctionSignature::from_function(
        &function.function,
        source_map,
      ))),
      (_, Some(annotation)) => Some(Self::Value(TypeScriptTypeRenderer::render_canonical(
        &annotation.type_ann,
        source_map,
      ))),
      (Some(Expr::TsAs(assertion)), None) => Some(Self::Value(TypeScriptTypeRenderer::render_canonical(
        &assertion.type_ann,
        source_map,
      ))),
      (Some(Expr::Object(object)), None) => Some(Self::from_object(object, source_map)),
      _ => None,
    }
  }

  /// An object literal's identifier- and string-keyed properties in declaration order, each typed by its value.
  fn from_object(object: &ObjectLit, source_map: &SourceMap) -> Self {
    Self::Object(
      object
        .props
        .iter()
        .filter_map(|property| {
          let Prop::KeyValue(property) = property.as_prop()?.as_ref() else {
            return None;
          };

          Some(TypeScriptObjectProperty {
            name: Self::read_property_name(&property.key)?,
            type_name: Self::infer_type(&property.value, source_map),
          })
        })
        .collect(),
    )
  }

  /// The type a value expression has without a checker: a function's, a nested object's, an asserted one or a
  /// literal's primitive, and `unknown` for anything else.
  fn infer_type(expression: &Expr, source_map: &SourceMap) -> String {
    match Self::unwrap_value(expression) {
      Expr::Arrow(arrow) => TypeScriptFunctionSignature::from_arrow(arrow, source_map).type_name(),
      Expr::Fn(function) => TypeScriptFunctionSignature::from_function(&function.function, source_map).type_name(),
      Expr::Object(object) => Self::from_object(object, source_map).type_name(),
      Expr::TsAs(assertion) => TypeScriptTypeRenderer::render_canonical(&assertion.type_ann, source_map),
      Expr::Lit(Lit::Bool(_)) => String::from("boolean"),
      Expr::Lit(Lit::Num(_)) => String::from("number"),
      Expr::Lit(Lit::Str(_)) => String::from("string"),
      _ => String::from(TypeScriptTypeRenderer::UNKNOWN),
    }
  }

  /// Strip the wrappers that keep a value's shape: `as const`, `satisfies` and parentheses.
  fn unwrap_value(expression: &Expr) -> &Expr {
    match expression {
      Expr::TsConstAssertion(assertion) => Self::unwrap_value(&assertion.expr),
      Expr::TsSatisfies(satisfies) => Self::unwrap_value(&satisfies.expr),
      Expr::Paren(parenthesized) => Self::unwrap_value(&parenthesized.expr),
      _ => expression,
    }
  }

  fn read_property_name(name: &PropName) -> Option<String> {
    match name {
      PropName::Ident(identifier) => Some(identifier.sym.to_string()),
      PropName::Str(value) => Some(value.value.to_string_lossy().to_string()),
      _ => None,
    }
  }
}
