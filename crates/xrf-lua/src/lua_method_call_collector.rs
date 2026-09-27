use std::collections::HashMap;

use full_moon::ast::{
  Ast, Call, Expression, FunctionArgs, FunctionCall, FunctionDeclaration, LocalAssignment, Prefix, Suffix,
};
use full_moon::node::Node;
use full_moon::visitors::Visitor;

use crate::xray_lua_chained_call::XRayLuaChainedCall;
use crate::xray_lua_method_call::XRayLuaMethodCall;
use crate::xray_lua_value::XRayLuaValue;

/// Where one named function begins and ends, so a call can say which one it was written in.
struct LuaFunctionRange {
  name: String,
  from: usize,
  to: usize,
}

/// A `local name = value` binding, and the line it was written on.
struct LuaLocalBinding {
  line: usize,
  name: String,
  value: XRayLuaValue,
}

/// What a script declares, as far as a reader that never runs it can collect it.
pub(crate) struct LuaCollected {
  pub(crate) method_calls: Vec<XRayLuaMethodCall>,
  /// Top-level locals bound to a literal string, by name, the last binding of a name winning as it would at run time.
  pub(crate) constants: HashMap<String, String>,
}

pub(crate) struct LuaMethodCallCollector {
  functions: Vec<LuaFunctionRange>,
  method_calls: Vec<XRayLuaMethodCall>,
  locals: Vec<LuaLocalBinding>,
}

impl LuaMethodCallCollector {
  pub(crate) fn collect(ast: &Ast) -> LuaCollected {
    let mut collector: Self = Self {
      functions: Vec::new(),
      locals: Vec::new(),
      method_calls: Vec::new(),
    };

    collector.visit_ast(ast);

    // Attributed by line range rather than by visiting order, so a call knows its function however the walk reached
    // it, and a file whose functions are declared after their calls still reads correctly.
    for call in &mut collector.method_calls {
      let line: usize = call.line_number();
      let function: Option<String> = collector
        .functions
        .iter()
        .find(|range| line >= range.from && line <= range.to)
        .map(|range| range.name.clone());

      call.set_function(function);
    }

    let mut constants: HashMap<String, String> = HashMap::new();

    for local in &collector.locals {
      let is_top_level: bool = !collector
        .functions
        .iter()
        .any(|range| local.line >= range.from && local.line <= range.to);

      if let (true, Some(value)) = (is_top_level, local.value.as_string()) {
        constants.insert(local.name.clone(), value.to_owned());
      }
    }

    LuaCollected {
      constants,
      method_calls: collector.method_calls,
    }
  }

  /// The arguments of one call, each read as far as a reader that never runs the script can read it.
  fn arguments(arguments: &FunctionArgs) -> Vec<XRayLuaValue> {
    match arguments {
      FunctionArgs::Parentheses { arguments, .. } => arguments.iter().map(XRayLuaValue::of).collect(),
      // `shader:begin "name"` is the same call written without its parentheses.
      FunctionArgs::String(string) => vec![XRayLuaValue::of(&Expression::String(string.clone()))],
      _ => vec![XRayLuaValue::Other],
    }
  }
}

impl Visitor for LuaMethodCallCollector {
  fn visit_local_assignment(&mut self, assignment: &LocalAssignment) {
    let Some(from) = assignment.start_position() else {
      return;
    };

    for (name, expression) in assignment.names().iter().zip(assignment.expressions().iter()) {
      self.locals.push(LuaLocalBinding {
        line: from.line(),
        name: name.token().to_string(),
        value: XRayLuaValue::of(expression),
      });
    }
  }

  fn visit_function_declaration(&mut self, declaration: &FunctionDeclaration) {
    let (Some(from), Some(to)) = (declaration.start_position(), declaration.end_position()) else {
      return;
    };

    self.functions.push(LuaFunctionRange {
      from: from.line(),
      name: declaration.name().to_string().trim().to_owned(),
      to: to.line(),
    });
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
          Self::arguments(call.args()),
        )),
        _ => None,
      })
      .collect();

    self.method_calls.push(XRayLuaMethodCall::from_parts(
      first.name().token().start_position().line(),
      receiver.token().to_string(),
      first.name().token().to_string(),
      Self::arguments(first.args()),
      chained,
    ));
  }
}
