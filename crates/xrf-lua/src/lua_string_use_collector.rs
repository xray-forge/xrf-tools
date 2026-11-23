use std::collections::HashSet;
use std::ptr;

use full_moon::ast::{
  Ast, BinOp, Call, Expression, Field, FunctionArgs, FunctionCall, Index, Prefix, Suffix, TableConstructor,
};
use full_moon::visitors::Visitor;

use crate::lua_scope::LuaScope;
use crate::xray_lua_string_call::XRayLuaStringCall;
use crate::xray_lua_value::XRayLuaValue;

/// Collects the literal strings a script uses as names: those it passes straight to a call, whatever it calls, those it
/// keeps in a table as data, and those it builds names from by concatenating something onto them.
///
/// `give_info_portion("name")`, `db.actor:has_info("name")`, `hasInfoPortion("name")` and `f "name"` pass a name,
/// `{ ordered = "name" }`, `{ [1] = "name" }` and `{ name = {...} }` keep one for later, and
/// `"zat_b51_ordered_item_" .. index` builds one. A field whose key spells its own value, `name = "name"`, only mirrors a name into a table of constants, such as
/// the one xrf-engine compiles its info portion names into, and names nothing here.
#[derive(Default)]
pub(crate) struct LuaStringUseCollector {
  /// The calls passed literal strings, each with the function it names, in source order.
  pub(crate) calls: Vec<XRayLuaStringCall>,
  /// Strings a table constructor holds as keys or values, in source order, a field whose key spells its value left out.
  pub(crate) table_strings: Vec<String>,
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

  /// Records one call to the function a name gives, when it passes any literal string.
  fn add_call(&mut self, function: Option<String>, arguments: &FunctionArgs) {
    let strings: Vec<String> = match arguments {
      FunctionArgs::Parentheses { arguments, .. } => arguments.iter().filter_map(Self::read_string).collect(),
      FunctionArgs::String(string) => Self::read_string(&Expression::String(string.clone()))
        .into_iter()
        .collect(),
      _ => Vec::new(),
    };

    if !strings.is_empty() {
      self.calls.push(XRayLuaStringCall::from_parts(function, strings));
    }
  }
}

impl Visitor for LuaStringUseCollector {
  /// Each call of a chain, named by what its spelling last named before it: the prefix of `f(...)`, the field of
  /// `a.b(...)`, or the method of `a:b(...)`. A bracketed index or a call's result names nothing.
  fn visit_function_call(&mut self, call: &FunctionCall) {
    let mut named: Option<String> = match call.prefix() {
      Prefix::Name(name) => Some(name.token().to_string()),
      _ => None,
    };

    for suffix in call.suffixes() {
      match suffix {
        Suffix::Index(Index::Dot { name, .. }) => named = Some(name.token().to_string()),
        Suffix::Call(Call::AnonymousCall(arguments)) => self.add_call(named.take(), arguments),
        Suffix::Call(Call::MethodCall(method)) => {
          self.add_call(Some(method.name().token().to_string()), method.args());
          named = None;
        }
        _ => named = None,
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

      let value: Option<String> = Self::read_string(value);

      // `name = "name"` mirrors a constant; any other field keeps its key and its string as data, as `supplies =
      // { supply_ammo_1 = {...} }` keys the portions a loop checks.
      if value.is_some() && key == value {
        continue;
      }

      self.table_strings.extend(key);
      self.table_strings.extend(value);
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
