use xrf_error::{XrfError, XrfResult};
use xrf_ini::IniSection;
use xrf_math::{Vector2d, Vector3d};

/// How the editor writes and reads its typed values: `CInifile::w_float`, `w_fvector2` and `w_fvector3` format with
/// `%f`, `w_u32` with `%d`, and `save_as` puts a space after every comma outside quotes (`_decorate`).
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

  /// The value a float reads back as once saved with [`Self::format_f32`], a zero of either sign as a positive one,
  /// which is how the editor's own files print it (`0.000000`).
  ///
  /// A transform decomposed from a compiled matrix is a few units in the last place off the angles the editor composed
  /// it from; rounded this way it composes into the same bits again, as the scene's text would.
  pub fn round_as_saved(value: f32) -> f32 {
    let saved: f32 = Self::format_f32(value).parse::<f32>().unwrap_or(value);

    if saved == 0.0 { 0.0 } else { saved }
  }

  /// The saved values nearest `value`, it rounded first, then a step either way and two, the step the coarser of the
  /// `%f` grid and the float's own: where a value is recovered from what the editor computed from it, the rounded one
  /// can land a step off the one it saved.
  pub fn get_saved_neighbours(value: f32) -> [f32; 5] {
    let step: f32 = (value.next_up() - value).max(1e-6);

    [0.0, 1.0, -1.0, 2.0, -2.0].map(|steps: f32| Self::round_as_saved(value + steps * step))
  }

  /// A vector rounded as [`Self::round_as_saved`] rounds each component.
  pub fn round_vector3_as_saved(value: &Vector3d) -> Vector3d {
    Vector3d::new(
      Self::round_as_saved(value.x),
      Self::round_as_saved(value.y),
      Self::round_as_saved(value.z),
    )
  }

  /// A vector as `w_fvector2` writes it and `_decorate` spaces it: `x, y`.
  pub fn format_vector2(value: &Vector2d) -> String {
    format!("{}, {}", Self::format_f32(value.x), Self::format_f32(value.y))
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

  /// An unsigned integer as `w_u32` writes it, through `%d`: a value past `i32::MAX` prints negative, as
  /// uninitialised flag bits in GSC's sound sources do.
  pub fn format_u32(value: u32) -> String {
    value.cast_signed().to_string()
  }

  /// A field `w_u32` wrote, read as `r_u32` reads it (`u32(atoi(...))`), a negative print standing for the same bits.
  ///
  /// # Errors
  ///
  /// Returns an error naming the section and key when the field is missing or not a 32-bit integer.
  pub fn read_u32(section: &IniSection, key: &str) -> XrfResult<u32> {
    let value: &str = section.get_required(key)?;

    value
      .parse::<i32>()
      .map(i32::cast_unsigned)
      .or_else(|_| value.parse::<u32>())
      .map_err(|_| {
        XrfError::new_parsing_error(format!(
          "Field [{}] {key} = '{value}' is not a 32-bit integer",
          section.name
        ))
      })
  }

  /// A field holding two comma-separated floats.
  ///
  /// # Errors
  ///
  /// Returns an error naming the section and key when the field is missing or not exactly two floats.
  pub fn read_vector2(section: &IniSection, key: &str) -> XrfResult<Vector2d> {
    Ok(Vector2d::from(Self::read_floats::<2>(section, key)?))
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
    let [x, y, z] = Self::read_floats::<3>(section, key)?;

    Ok(Vector3d { x, y, z })
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

  /// A field holding exactly `N` comma-separated floats.
  fn read_floats<const N: usize>(section: &IniSection, key: &str) -> XrfResult<[f32; N]> {
    let value: &str = section.get_required(key)?;

    value
      .split(',')
      .map(|component| component.trim().parse::<f32>().ok())
      .collect::<Option<Vec<f32>>>()
      .and_then(|components| <[f32; N]>::try_from(components).ok())
      .ok_or_else(|| {
        XrfError::new_parsing_error(format!("Field [{}] {key} = '{value}' is not {N} floats", section.name))
      })
  }
}
