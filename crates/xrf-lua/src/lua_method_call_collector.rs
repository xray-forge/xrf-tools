use std::collections::HashSet;
use std::ptr;

use full_moon::ast::{
  Assignment, Ast, Block, Call, Expression, FunctionArgs, FunctionBody, FunctionCall, FunctionDeclaration,
  FunctionName, GenericFor, LocalAssignment, LocalFunction, NumericFor, Parameter, Prefix, Repeat, Suffix, Var,
};
use full_moon::tokenizer::TokenReference;
use full_moon::visitors::Visitor;

use crate::lua_scope::LuaScope;
use crate::xray_lua_binding::XRayLuaBinding;
use crate::xray_lua_chained_call::XRayLuaChainedCall;
use crate::xray_lua_method_call::XRayLuaMethodCall;
use crate::xray_lua_value::XRayLuaValue;

/// Collects a script's method calls, each argument read with the scope it is written in.
///
/// A scope opens where Lua's does: a block, a function body, a loop's body with its variables, and a `repeat` body
/// whose locals its `until` still sees. A local some assignment gives another value reads as unknown throughout.
#[derive(Default)]
pub(crate) struct LuaMethodCallCollector {
  method_calls: Vec<XRayLuaMethodCall>,
  scope: LuaScope,
  /// The named functions the walk is inside, the innermost last.
  functions: Vec<String>,
  /// Declarations an assignment gives another value, by where each is written, found by the first walk.
  reassigned: HashSet<usize>,
  /// The body of each loop being entered and the variables it declares there, which its bounds do not see.
  loops: Vec<(*const Block, Vec<(String, usize)>)>,
  /// The body of each `repeat` being walked, whose frame stays open for its `until`.
  repeats: Vec<*const Block>,
  /// Where the method a declaration names is written, whose body binds `self` there.
  method: Option<usize>,
}

impl LuaMethodCallCollector {
  /// Every method call of a script, in source order: a first walk finds the locals assignments change, and the
  /// second reads the calls knowing them.
  pub(crate) fn collect(ast: &Ast) -> Vec<XRayLuaMethodCall> {
    let mut first: Self = Self::default();

    first.visit_ast(ast);

    let mut collector: Self = Self {
      scope: LuaScope::of_reassigned(first.reassigned),
      ..Self::default()
    };

    collector.visit_ast(ast);
    collector.method_calls
  }

  /// Where a name token is written, which is what tells one declaration of a name from another.
  fn position(token: &TokenReference) -> usize {
    token.token().start_position().bytes()
  }

  /// Notes that an assignment gives the declaration a name means here another value.
  fn reassign(&mut self, name: &TokenReference) {
    if let Some(at) = self.scope.resolve_declaration(&name.token().to_string()) {
      self.reassigned.insert(at);
    }
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
  fn visit_block(&mut self, block: &Block) {
    self.scope.push();

    // A loop's variables are declared in its body, after its bounds were read outside them.
    if self.loops.last().is_some_and(|(body, _)| ptr::eq(*body, block))
      && let Some((_, variables)) = self.loops.pop()
    {
      for (name, at) in variables {
        self.scope.bind(name, at, XRayLuaBinding::Other);
      }
    }
  }

  fn visit_block_end(&mut self, block: &Block) {
    // A `repeat` body's locals are in scope for its `until`, which is walked after the body.
    if self.repeats.last().is_some_and(|body| ptr::eq(*body, block)) {
      return;
    }

    self.scope.pop();
  }

  fn visit_repeat(&mut self, repeat: &Repeat) {
    self.repeats.push(ptr::from_ref(repeat.block()));
  }

  fn visit_repeat_end(&mut self, _: &Repeat) {
    self.repeats.pop();
    self.scope.pop();
  }

  fn visit_function_body(&mut self, body: &FunctionBody) {
    self.scope.push();

    // `function object:method()` is `function object.method(self)`.
    if let Some(at) = self.method.take() {
      self.scope.bind(String::from("self"), at, XRayLuaBinding::Parameter);
    }

    for parameter in body.parameters() {
      if let Parameter::Name(name) = parameter {
        self.scope.bind(
          name.token().to_string(),
          Self::position(name),
          XRayLuaBinding::Parameter,
        );
      }
    }
  }

  fn visit_function_body_end(&mut self, _: &FunctionBody) {
    self.scope.pop();
  }

  fn visit_numeric_for(&mut self, numeric_for: &NumericFor) {
    let variable: &TokenReference = numeric_for.index_variable();

    self.loops.push((
      ptr::from_ref(numeric_for.block()),
      vec![(variable.token().to_string(), Self::position(variable))],
    ));
  }

  fn visit_generic_for(&mut self, generic_for: &GenericFor) {
    self.loops.push((
      ptr::from_ref(generic_for.block()),
      generic_for
        .names()
        .iter()
        .map(|name| (name.token().to_string(), Self::position(name)))
        .collect(),
    ));
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
        Self::position(name),
        values.get(index).cloned().unwrap_or(XRayLuaBinding::Other),
      );
    }
  }

  // Before its body: a local function sees itself.
  fn visit_local_function(&mut self, function: &LocalFunction) {
    self.scope.bind(
      function.name().token().to_string(),
      Self::position(function.name()),
      XRayLuaBinding::Other,
    );
  }

  fn visit_assignment(&mut self, assignment: &Assignment) {
    for variable in assignment.variables() {
      if let Var::Name(name) = variable {
        self.reassign(name);
      }
    }
  }

  fn visit_function_declaration(&mut self, declaration: &FunctionDeclaration) {
    let name: &FunctionName = declaration.name();

    match name.method_name() {
      Some(method) => self.method = Some(Self::position(method)),
      // `function name()` assigns the local `name` where one is in scope.
      None if name.names().len() == 1 => {
        if let Some(only) = name.names().iter().next() {
          self.reassign(only);
        }
      }
      None => {}
    }

    self.functions.push(name.to_string().trim().to_owned());
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
