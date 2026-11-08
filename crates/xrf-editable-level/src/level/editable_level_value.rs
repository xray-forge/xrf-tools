use xrf_error::{XrfError, XrfResult};
use xrf_ini::IniSection;
use xrf_math::Vector3d;

/// How the editor writes and reads its typed values: `CInifile::w_float` and `w_fvector3` format with `%f`, and
/// `save_as` puts a space after every comma outside quotes (`_decorate`).
///
/// Not every number a saved scene holds is an `f32` printed with `%f`: `labx8.level`'s camera stands at `-80.052984`,
/// which no `f32` prints as. A typed read is therefore close to the text rather than equal to it, and a file writes
/// back byte for byte through the text its `IniFile` keeps, not through typed values.
pub struct EditableLevelValue;

impl EditableLevelValue {
  /// A float as `%f` writes it: six decimals, no exponent.
  pub fn format_f32(value: f32) -> String {
    format!("{value:.6}")
  }

  /// A vector as `w_fvector3` writes it and `_decorate` spaces it: `x, y, z`.
  pub fn format_vector3(value: &Vector3d) -> String {
    format!(
      "{}, {}, {}",
      Self::format_f32(value.x),
      Self::format_f32(value.y),
      Self::format_f32(value.z)
    )
  }

  /// A string as the editor writes a quoted one, such as the build options' text: `"text"`.
  pub fn format_quoted(value: &str) -> String {
    format!("\"{value}\"")
  }

  /// A field holding three comma-separated floats.
  ///
  /// # Errors
  ///
  /// Returns an error naming the section and key when the field is missing or not exactly three floats.
  pub fn read_vector3(section: &IniSection, key: &str) -> XrfResult<Vector3d> {
    let value: &str = section.get_required(key)?;
    let components: Vec<f32> = value
      .split(',')
      .map(|component| component.trim().parse::<f32>())
      .collect::<Result<Vec<f32>, _>>()
      .map_err(|_| Self::vector_error(&section.name, key, value))?;

    match components.as_slice() {
      [x, y, z] => Ok(Vector3d { x: *x, y: *y, z: *z }),
      _ => Err(Self::vector_error(&section.name, key, value)),
    }
  }

  /// A field holding a quoted string, without its quotes.
  ///
  /// # Errors
  ///
  /// Returns an error naming the section and key when the field is missing or not one quoted string.
  pub fn read_quoted<'a>(section: &'a IniSection, key: &str) -> XrfResult<&'a str> {
    let value: &str = section.get_required(key)?;

    value
      .strip_prefix('"')
      .and_then(|value| value.strip_suffix('"'))
      .filter(|value| !value.contains('"'))
      .ok_or_else(|| {
        XrfError::new_parsing_error(format!(
          "Field [{}] {key} = '{value}' is not one quoted string",
          section.name
        ))
      })
  }

  fn vector_error(section: &str, key: &str, value: &str) -> XrfError {
    XrfError::new_parsing_error(format!("Field [{section}] {key} = '{value}' is not three floats"))
  }
}
