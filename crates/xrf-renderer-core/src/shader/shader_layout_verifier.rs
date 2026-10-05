use xrf_error::{XrfError, XrfResult};

use crate::shader::shader_address_space::ShaderAddressSpace;
use crate::shader::shader_declarations::ShaderDeclarations;
use crate::shader::shader_struct::ShaderStruct;

/// Checks a shared struct against a second, independent reading of WGSL: naga parses the declaration the derive wrote,
/// lays it out itself, and validates it bound in its address space.
pub struct ShaderLayoutVerifier;

impl ShaderLayoutVerifier {
  /// Passes when naga places every member where the derive did, makes the struct the same size, and accepts it bound in
  /// `space`.
  ///
  /// # Errors
  ///
  /// Returns an error naming the first member, size or rule that disagrees.
  pub fn verify<T: ShaderStruct>(space: ShaderAddressSpace) -> XrfResult {
    let name: String = T::get_wgsl_name();
    let source: String = format!(
      "{}\n@group(0) @binding(0) var<{}> probe: {name};\n",
      ShaderDeclarations::new().declare::<T>().to_wgsl(),
      space.get_wgsl()
    );
    let module: naga::Module = naga::front::wgsl::parse_str(&source).map_err(|error| {
      XrfError::new_invalid_error(format!(
        "naga cannot parse `{name}`'s declaration: {}",
        error.emit_to_string(&source)
      ))
    })?;

    naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::all())
      .validate(&module)
      .map_err(|error| {
        XrfError::new_invalid_error(format!(
          "naga refuses `{name}` bound as {space:?}: {}",
          error.as_inner()
        ))
      })?;

    let (members, span) = module
      .types
      .iter()
      .find_map(|(_, ty)| match &ty.inner {
        naga::TypeInner::Struct { members, span } if ty.name.as_deref() == Some(name.as_str()) => {
          Some((members, *span))
        }
        _ => None,
      })
      .ok_or_else(|| XrfError::new_unexpected_error(format!("naga parsed no struct `{name}`")))?;

    if members.len() != T::MEMBERS.len() {
      return Err(XrfError::new_invalid_error(format!(
        "naga reads {} members of `{name}` where the derive has {}",
        members.len(),
        T::MEMBERS.len()
      )));
    }

    for (parsed, member) in members.iter().zip(T::MEMBERS) {
      if parsed.name.as_deref() != Some(member.name) || u64::from(parsed.offset) != member.offset {
        return Err(XrfError::new_invalid_error(format!(
          "naga places `{name}.{}` at {} where the derive places `{}` at {}",
          parsed.name.as_deref().unwrap_or("?"),
          parsed.offset,
          member.name,
          member.offset
        )));
      }
    }

    if u64::from(span) != T::SIZE {
      return Err(XrfError::new_invalid_error(format!(
        "naga makes `{name}` {span} bytes where the derive makes it {}",
        T::SIZE
      )));
    }

    Ok(())
  }
}
