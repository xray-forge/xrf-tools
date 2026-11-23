use std::collections::HashSet;
use std::ptr;

use full_moon::ast::{Ast, BinOp, Call, Expression, Field, FunctionArgs, FunctionCall, Suffix, TableConstructor};
use full_moon::visitors::Visitor;

use crate::lua_scope::LuaScope;
use crate::xray_lua_value::XRayLuaValue;

/// Collects the literal strings a script uses as names: those it passes straight to a call, whatever it calls, those it
/// keeps in a table as data, and those it builds names from by concatenating something onto them.
///
/// `give_info_portion("name")`, `db.actor:has_info("name")`, `hasInfoPortion("name")` and `f "name"` pass a name,
/// `{ ordered = "name" }` and `{ [1] = "name" }` keep one for a later call, and `"zat_b51_ordered_item_" .. index` builds
/// one. A field whose key spells its own value, `name = "name"`, only mirrors a name into a table of constants, such as
/// the one xrf-engine compiles its info portion names into, and names nothing here.
#[derive(Default)]
pub(crate) struct LuaStringUseCollector {
  /// Strings passed straight to a call, in source order.
  pub(crate) call_arguments: Vec<String>,
  /// Strings a table constructor holds as values under another key or none, in source order.
  pub(crate) table_values: Vec<String>,
  /// Strings opening a chain of `..`, in source order.
  pub(crate) concatenation_prefixes: Vec<String>,
  /// The right sides of the `..` met so far, which continue a chain rather than open one.
  concatenation_tails: HashSet<*const Expression>,
}

impl LuaStringUseCollector {
  pub(crate) fn collect(ast: &Ast) -> Self {
    let mut collector: Self = Self::default();

    collector.visit_ast(ast);
    collector
  }

  /// The string an expression is, for a literal one.
  fn read_string(expression: &Expression) -> Option<String> {
    XRayLuaValue::of(expression, &LuaScope::default())
      .as_string()
      .map(str::to_owned)
  }

  /// The literal strings among one call's arguments.
  fn add_arguments(&mut self, arguments: &FunctionArgs) {
    match arguments {
      FunctionArgs::Parentheses { arguments, .. } => {
        self
          .call_arguments
          .extend(arguments.iter().filter_map(Self::read_string));
      }
      FunctionArgs::String(string) => {
        self
          .call_arguments
          .extend(Self::read_string(&Expression::String(string.clone())));
      }
      _ => {}
    }
  }
}

impl Visitor for LuaStringUseCollector {
  fn visit_function_call(&mut self, call: &FunctionCall) {
    for suffix in call.suffixes() {
      match suffix {
        Suffix::Call(Call::AnonymousCall(arguments)) => self.add_arguments(arguments),
        Suffix::Call(Call::MethodCall(method)) => self.add_arguments(method.args()),
        _ => {}
      }
    }
  }

  fn visit_table_constructor(&mut self, table: &TableConstructor) {
    for field in table.fields() {
      let value: &Expression = match field {
        Field::NameKey { value, .. } | Field::ExpressionKey { value, .. } => value,
        Field::NoKey(value) => value,
        _ => continue,
      };
      let key: Option<String> = match field {
        Field::NameKey { key, .. } => Some(key.token().to_string()),
        Field::ExpressionKey { key, .. } => Self::read_string(key),
        _ => None,
      };

      if let Some(value) = Self::read_string(value)
        && key.as_deref() != Some(value.as_str())
      {
        self.table_values.push(value);
      }
    }
  }

  // `..` is right associative, so the literal opening `"a" .. b .. "c"` is the left side of the outermost one; an inner
  // one, the right side of another, starts in the middle of the name and opens nothing.
  fn visit_expression(&mut self, expression: &Expression) {
    let Expression::BinaryOperator {
      lhs,
      binop: BinOp::TwoDots(_),
      rhs,
    } = expression
    else {
      return;
    };

    // Visited before its parts, so its right side is marked before the walk reaches it.
    self.concatenation_tails.insert(ptr::from_ref(rhs.as_ref()));

    if !self.concatenation_tails.contains(&ptr::from_ref(expression)) {
      self.concatenation_prefixes.extend(Self::read_string(lhs));
    }
  }
}
