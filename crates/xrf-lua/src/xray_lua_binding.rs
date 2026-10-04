/// What a bare name means where a script uses it, by Lua's own scoping: the innermost local or parameter of that name.
#[derive(Clone, Debug, PartialEq)]
pub enum XRayLuaBinding {
  /// A local bound to a literal string, `local tex_base = "water\\water_water"`.
  String(String),
  /// A parameter of an enclosing function, whose value the function's caller supplies: the argument at `index`, the
  /// implicit `self` of a method being the first.
  Parameter { index: usize },
  /// A local bound to anything else.
  Other,
}
