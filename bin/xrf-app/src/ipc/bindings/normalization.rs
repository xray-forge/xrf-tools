use xrf_typescript::ast::ts_type_to_string;
use xrf_typescript::swc_common::DUMMY_SP;
use xrf_typescript::swc_ecma_ast::{
  ArrayLit, BlockStmtOrExpr, Decl, Expr, Lit, ModuleDecl, ModuleItem, Pat, Prop, PropName, PropOrSpread, TsArrayType,
  TsEntityName, TsFnOrConstructorType, TsFnParam, TsType, TsTypeElement, TsUnionOrIntersectionType,
};

/// Rewrite a declaration so only a change of shape can change how it renders.
pub(super) fn normalize_module_item(item: &mut ModuleItem) {
  if let ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(export)) = item {
    normalize_declaration(&mut export.decl);
  }
}

fn normalize_declaration(declaration: &mut Decl) {
  match declaration {
    Decl::TsTypeAlias(alias) => normalize_type(&mut alias.type_ann),
    Decl::TsInterface(interface) => normalize_type_elements(&mut interface.body.body),
    Decl::Var(variable) => {
      for declarator in &mut variable.decls {
        if let Some(initializer) = &mut declarator.init {
          normalize_expression(initializer);
        }
      }
    }
    _ => {}
  }
}

fn normalize_type(ts_type: &mut TsType) {
  // Collapsed before the match so the node underneath is normalized like any other.
  if let TsType::TsParenthesizedType(parenthesized) = ts_type {
    *ts_type = (*parenthesized.type_ann).clone();

    return normalize_type(ts_type);
  }

  if let Some(element) = generic_array_element(ts_type) {
    *ts_type = TsType::TsArrayType(TsArrayType {
      span: DUMMY_SP,
      elem_type: Box::new(element),
    });
  }

  match ts_type {
    TsType::TsArrayType(array) => normalize_type(&mut array.elem_type),
    TsType::TsOptionalType(optional) => normalize_type(&mut optional.type_ann),
    TsType::TsRestType(rest) => normalize_type(&mut rest.type_ann),
    TsType::TsTupleType(tuple) => {
      // Tuple order is the one ordering that is part of the type, so it is left alone.
      for element in &mut tuple.elem_types {
        normalize_type(&mut element.ty);
      }
    }
    TsType::TsTypeLit(literal) => normalize_type_elements(&mut literal.members),
    // A map crosses as `{ [key in K]: V }`, so its value carries the same rewrites as any other type.
    TsType::TsMappedType(mapped) => {
      if let Some(name) = &mut mapped.name_type {
        normalize_type(name);
      }

      if let Some(annotation) = &mut mapped.type_ann {
        normalize_type(annotation);
      }
    }
    TsType::TsIndexedAccessType(indexed) => {
      normalize_type(&mut indexed.obj_type);
      normalize_type(&mut indexed.index_type);
    }
    TsType::TsTypeOperator(operator) => normalize_type(&mut operator.type_ann),
    TsType::TsUnionOrIntersectionType(composite) => {
      let members = match composite {
        TsUnionOrIntersectionType::TsUnionType(union) => &mut union.types,
        TsUnionOrIntersectionType::TsIntersectionType(intersection) => &mut intersection.types,
      };

      for member in members.iter_mut() {
        normalize_type(member);
      }

      members.sort_by_key(|member| type_sort_key(member));
    }
    TsType::TsTypeRef(reference) => {
      if let Some(parameters) = &mut reference.type_params {
        for parameter in &mut parameters.params {
          normalize_type(parameter);
        }
      }
    }
    TsType::TsFnOrConstructorType(TsFnOrConstructorType::TsFnType(function)) => {
      for parameter in &mut function.params {
        if let TsFnParam::Ident(binding) = parameter
          && let Some(annotation) = &mut binding.type_ann
        {
          normalize_type(&mut annotation.type_ann);
        }
      }

      normalize_type(&mut function.type_ann.type_ann);
    }
    _ => {}
  }
}

fn normalize_type_elements(members: &mut [TsTypeElement]) {
  for member in members.iter_mut() {
    if let TsTypeElement::TsPropertySignature(property) = member
      && let Some(annotation) = &mut property.type_ann
    {
      normalize_type(&mut annotation.type_ann);
    }
  }

  members.sort_by_key(type_element_name);
}

fn normalize_expression(expression: &mut Expr) {
  match expression {
    Expr::Paren(parenthesized) => {
      *expression = (*parenthesized.expr).clone();

      normalize_expression(expression);
    }
    Expr::Object(object) => {
      for property in &mut object.props {
        if let PropOrSpread::Prop(declared) = property
          && let Prop::KeyValue(entry) = declared.as_mut()
        {
          normalize_expression(&mut entry.value);
        }
      }

      object.props.sort_by_key(property_name);
    }
    Expr::Array(ArrayLit { elems, .. }) => {
      for element in elems.iter_mut().flatten() {
        normalize_expression(&mut element.expr);
      }
    }
    Expr::Arrow(arrow) => {
      for parameter in &mut arrow.params {
        if let Pat::Ident(binding) = parameter
          && let Some(annotation) = &mut binding.type_ann
        {
          normalize_type(&mut annotation.type_ann);
        }
      }

      if let Some(annotation) = &mut arrow.return_type {
        normalize_type(&mut annotation.type_ann);
      }

      if let BlockStmtOrExpr::Expr(body) = arrow.body.as_mut() {
        normalize_expression(body);
      }
    }
    Expr::Call(call) => {
      if let Some(parameters) = &mut call.type_args {
        for parameter in &mut parameters.params {
          normalize_type(parameter);
        }
      }

      for argument in &mut call.args {
        normalize_expression(&mut argument.expr);
      }
    }
    _ => {}
  }
}

/// The element type of an `Array<T>` reference, which is the frontend's spelling of `T[]`.
fn generic_array_element(ts_type: &TsType) -> Option<TsType> {
  let TsType::TsTypeRef(reference) = ts_type else {
    return None;
  };
  let TsEntityName::Ident(name) = &reference.type_name else {
    return None;
  };

  if name.sym != *"Array" {
    return None;
  }

  let parameters = reference.type_params.as_ref()?;

  match parameters.params.as_slice() {
    [element] => Some((**element).clone()),
    _ => None,
  }
}

/// Total order over the members of a union or intersection, stable across both spellings of a shape.
fn type_sort_key(ts_type: &TsType) -> String {
  match ts_type {
    // `ts_type_to_string` renders a type literal as `unsupported`, so its members carry the key instead.
    TsType::TsTypeLit(literal) => format!(
      "lit:{}",
      literal
        .members
        .iter()
        .map(type_element_name)
        .collect::<Vec<String>>()
        .join(",")
    ),
    TsType::TsUnionOrIntersectionType(composite) => {
      let members = match composite {
        TsUnionOrIntersectionType::TsUnionType(union) => &union.types,
        TsUnionOrIntersectionType::TsIntersectionType(intersection) => &intersection.types,
      };

      format!(
        "composite:{}",
        members
          .iter()
          .map(|member| type_sort_key(member))
          .collect::<Vec<String>>()
          .join("&")
      )
    }
    other => ts_type_to_string(other),
  }
}

/// Declared name of a type member, which is what a frontend property access resolves through.
fn type_element_name(member: &TsTypeElement) -> String {
  let TsTypeElement::TsPropertySignature(property) = member else {
    return String::new();
  };

  match property.key.as_ref() {
    Expr::Ident(identifier) => identifier.sym.to_string(),
    Expr::Lit(Lit::Str(literal)) => literal.value.to_string_lossy().to_string(),
    _ => String::new(),
  }
}

fn property_name(property: &PropOrSpread) -> String {
  let PropOrSpread::Prop(declared) = property else {
    return String::new();
  };

  match declared.as_ref() {
    Prop::Shorthand(identifier) => identifier.sym.to_string(),
    Prop::KeyValue(entry) => match &entry.key {
      PropName::Ident(identifier) => identifier.sym.to_string(),
      PropName::Str(literal) => literal.value.to_string_lossy().to_string(),
      _ => String::new(),
    },
    _ => String::new(),
  }
}
