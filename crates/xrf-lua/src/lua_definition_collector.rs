use full_moon::ast::{
  Assignment, Ast, Call, Expression, Field, FunctionArgs, FunctionCall, FunctionDeclaration, FunctionName, Prefix,
  Suffix, Var,
};
use full_moon::visitors::Visitor;

use crate::lua_scope::LuaScope;
use crate::xray_lua_value::XRayLuaValue;

/// Collects the functions a script defines for others to call by name: every non-local function it declares or
/// assigns to a plain name, and the names it registers through xrf-engine's `extern(name, value)`.
///
/// X-Ray runs a script in the namespace its file names, so every name it does not declare `local` lands there, a
/// function a body defines as well as one the top level does. A name an earlier `local` declares is still counted: the
/// list answers what may be defined, and a caller judging a missing name must not fail one that is.
#[derive(Default)]
pub(crate) struct LuaDefinitionCollector {
  /// Names of the non-local functions the script declares or assigns, in source order.
  pub(crate) global_functions: Vec<String>,
  /// Literal names passed to `extern`, wherever the call is written, in source order.
  pub(crate) registered_names: Vec<String>,
}

impl LuaDefinitionCollector {
  /// The function xrf-engine's runtime library assigns a value to a dotted global path with.
  const REGISTER_FUNCTION: &'static str = "extern";

  pub(crate) fn collect(ast: &Ast) -> Self {
    let mut collector: Self = Self::default();

    collector.visit_ast(ast);
    collector
  }
}

impl Visitor for LuaDefinitionCollector {
  /// `function name()`; a method or a dotted name defines a member of another table, not a name of the namespace.
  fn visit_function_declaration(&mut self, declaration: &FunctionDeclaration) {
    let name: &FunctionName = declaration.name();

    if name.method_name().is_none()
      && name.names().len() == 1
      && let Some(only) = name.names().iter().next()
    {
      self.global_functions.push(only.token().to_string());
    }
  }

  /// `name = function() end`, each name its own function is assigned to.
  fn visit_assignment(&mut self, assignment: &Assignment) {
    for (variable, expression) in assignment.variables().iter().zip(assignment.expressions().iter()) {
      if let (Var::Name(name), Expression::Function(_)) = (variable, expression) {
        self.global_functions.push(name.token().to_string());
      }
    }
  }

  fn visit_function_call(&mut self, call: &FunctionCall) {
    let Prefix::Name(function) = call.prefix() else {
      return;
    };

    if function.token().to_string() != Self::REGISTER_FUNCTION {
      return;
    }

    let Some(Suffix::Call(Call::AnonymousCall(arguments))) = call.suffixes().next() else {
      return;
    };
    let FunctionArgs::Parentheses { arguments, .. } = arguments.as_ref() else {
      return;
    };

    let mut arguments = arguments.iter();
    let scope: LuaScope = LuaScope::default();

    // Only a literal name is known without running the script.
    let Some(name) = arguments
      .next()
      .map(|first| XRayLuaValue::of(first, &scope))
      .and_then(|name| name.as_string().map(str::to_owned))
    else {
      return;
    };

    // `extern(name, { member = ... })` registers the table under the name, and with it each member it is written with.
    if let Some(Expression::TableConstructor(table)) = arguments.next() {
      for field in table.fields() {
        let member: Option<String> = match field {
          Field::NameKey { key, .. } => Some(key.token().to_string()),
          Field::ExpressionKey { key, .. } => XRayLuaValue::of(key, &scope).as_string().map(str::to_owned),
          _ => None,
        };

        self
          .registered_names
          .extend(member.map(|member| format!("{name}.{member}")));
      }
    }

    self.registered_names.push(name);
  }
}
