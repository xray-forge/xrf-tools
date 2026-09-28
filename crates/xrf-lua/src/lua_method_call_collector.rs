use full_moon::ast::{
  Ast, Block, Call, Expression, FunctionArgs, FunctionBody, FunctionCall, FunctionDeclaration, GenericFor,
  LocalAssignment, LocalFunction, NumericFor, Parameter, Prefix, Suffix,
};
use full_moon::visitors::Visitor;

use crate::lua_scope::LuaScope;
use crate::xray_lua_binding::XRayLuaBinding;
use crate::xray_lua_chained_call::XRayLuaChainedCall;
use crate::xray_lua_method_call::XRayLuaMethodCall;
use crate::xray_lua_value::XRayLuaValue;

/// Collects a script's method calls in one walk, each argument read with the scope it is written in.
///
/// A scope opens where Lua's does, at a block, a function body and a loop, so a parameter or a local shadowing a
/// top-level name reads as what it is. The one departure: a loop's variables are in scope for its own bounds.
#[derive(Default)]
pub(crate) struct LuaMethodCallCollector {
  method_calls: Vec<XRayLuaMethodCall>,
  scope: LuaScope,
  /// The named functions the walk is inside, the innermost last.
  functions: Vec<String>,
}

impl LuaMethodCallCollector {
  /// Every method call of a script, in source order.
  pub(crate) fn collect(ast: &Ast) -> Vec<XRayLuaMethodCall> {
    let mut collector: Self = Self::default();

    collector.visit_ast(ast);
    collector.method_calls
  }

  /// The arguments of one call, each read as far as a reader that never runs the script can read it.
  fn arguments(&self, arguments: &FunctionArgs) -> Vec<XRayLuaValue> {
    match arguments {
      FunctionArgs::Parentheses { arguments, .. } => arguments
        .iter()
        .map(|argument| XRayLuaValue::of(argument, &self.scope))
        .collect(),
      // `shader:begin "name"` is the same call written without its parentheses.
      FunctionArgs::String(string) => vec![XRayLuaValue::of(&Expression::String(string.clone()), &self.scope)],
      _ => vec![XRayLuaValue::Other],
    }
  }
}

impl Visitor for LuaMethodCallCollector {
  fn visit_block(&mut self, _: &Block) {
    self.scope.push();
  }

  fn visit_block_end(&mut self, _: &Block) {
    self.scope.pop();
  }

  fn visit_function_body(&mut self, body: &FunctionBody) {
    self.scope.push();

    for parameter in body.parameters() {
      if let Parameter::Name(name) = parameter {
        self.scope.bind(name.token().to_string(), XRayLuaBinding::Parameter);
      }
    }
  }

  fn visit_function_body_end(&mut self, _: &FunctionBody) {
    self.scope.pop();
  }

  fn visit_numeric_for(&mut self, numeric_for: &NumericFor) {
    self.scope.push();
    self
      .scope
      .bind(numeric_for.index_variable().token().to_string(), XRayLuaBinding::Other);
  }

  fn visit_numeric_for_end(&mut self, _: &NumericFor) {
    self.scope.pop();
  }

  fn visit_generic_for(&mut self, generic_for: &GenericFor) {
    self.scope.push();

    for name in generic_for.names() {
      self.scope.bind(name.token().to_string(), XRayLuaBinding::Other);
    }
  }

  fn visit_generic_for_end(&mut self, _: &GenericFor) {
    self.scope.pop();
  }

  // After its values are read: `local x = x` reads the `x` outside.
  fn visit_local_assignment_end(&mut self, assignment: &LocalAssignment) {
    let values: Vec<XRayLuaBinding> = assignment
      .expressions()
      .iter()
      .map(|expression| XRayLuaValue::of(expression, &self.scope).to_binding())
      .collect();

    for (index, name) in assignment.names().iter().enumerate() {
      self.scope.bind(
        name.token().to_string(),
        values.get(index).cloned().unwrap_or(XRayLuaBinding::Other),
      );
    }
  }

  // Before its body: a local function sees itself.
  fn visit_local_function(&mut self, function: &LocalFunction) {
    self
      .scope
      .bind(function.name().token().to_string(), XRayLuaBinding::Other);
  }

  fn visit_function_declaration(&mut self, declaration: &FunctionDeclaration) {
    self.functions.push(declaration.name().to_string().trim().to_owned());
  }

  fn visit_function_declaration_end(&mut self, _: &FunctionDeclaration) {
    self.functions.pop();
  }

  fn visit_function_call(&mut self, function_call: &FunctionCall) {
    let Prefix::Name(receiver) = function_call.prefix() else {
      return;
    };

    let mut suffixes = function_call.suffixes();

    let Some(Suffix::Call(Call::MethodCall(first))) = suffixes.next() else {
      return;
    };

    // Everything after the first suffix is chained onto what the one before it answered, which is how a script states
    // a pass: one `begin` and then the states it is drawn with.
    let chained: Vec<XRayLuaChainedCall> = suffixes
      .filter_map(|suffix| match suffix {
        Suffix::Call(Call::MethodCall(call)) => Some(XRayLuaChainedCall::from_parts(
          call.name().token().to_string(),
          self.arguments(call.args()),
        )),
        _ => None,
      })
      .collect();
    let mut call: XRayLuaMethodCall = XRayLuaMethodCall::from_parts(
      first.name().token().start_position().line(),
      receiver.token().to_string(),
      first.name().token().to_string(),
      self.arguments(first.args()),
      chained,
    );

    call.set_function(self.functions.last().cloned());
    self.method_calls.push(call);
  }
}
