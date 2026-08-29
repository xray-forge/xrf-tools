use xrf_error::{XrfError, XrfResult};

/// Assert a declared record count fits the budget the records will be read from.
///
/// A count read out of a file is untrusted: a truncated or hostile one can declare four billion records in a payload
/// that holds none, and reserving from it aborts the process instead of failing the parse. `budget` and
/// `min_record_size` share a unit - bytes for a binary reader, sections for an ltx one - and the minimum counts only
/// what every record consumes unconditionally, so understating it stays correct while overstating it rejects valid
/// input.
pub fn assert_count_fits(count: u64, budget: u64, min_record_size: u64, what: &str) -> XrfResult {
  let max_count: u64 = budget / min_record_size.max(1);

  if count > max_count {
    return Err(XrfError::new_invalid_error(format!(
      "{what} declares {count} entries of at least {min_record_size} each, but only {budget} remain"
    )));
  }

  Ok(())
}

/// Create a vector holding a declared record count, proving the count against its budget before reserving it.
///
/// See [`assert_count_fits`] for what the budget means. The reservation itself is fallible, so a count that survives
/// the bound but cannot be satisfied by the host is reported rather than aborting the process.
pub fn new_bounded_vec<T>(count: u64, budget: u64, min_record_size: u64, what: &str) -> XrfResult<Vec<T>> {
  assert_count_fits(count, budget, min_record_size, what)?;

  let capacity: usize = usize::try_from(count)
    .map_err(|_| XrfError::new_invalid_error(format!("{what} declares {count} entries, over the platform limit")))?;

  let mut vector: Vec<T> = Vec::new();

  vector
    .try_reserve_exact(capacity)
    .map_err(|error| XrfError::new_read_error(format!("Cannot reserve {count} entries for {what}: {error}")))?;

  Ok(vector)
}

/// A fixed-width format field a length can be narrowed into.
///
/// The label is carried here rather than read from [`std::any::type_name`], whose output the standard library does not
/// guarantee, because it reaches the user inside an error message.
pub trait FormatWidth: Sized + TryFrom<usize> {
  const LABEL: &'static str;
}

impl FormatWidth for u8 {
  const LABEL: &'static str = "u8";
}

impl FormatWidth for u16 {
  const LABEL: &'static str = "u16";
}

impl FormatWidth for u32 {
  const LABEL: &'static str = "u32";
}

/// Narrow an in-memory length into the fixed-width field its format stores it in.
///
/// A binary format states a count or a size in a field of a fixed width, and an `as` cast into that width truncates
/// silently: the file is then written with a length that does not describe its own payload, and the defect only
/// surfaces when something reads it back. `what` names the collection or payload being written.
pub fn to_format_size<T: FormatWidth>(value: usize, what: &str) -> XrfResult<T> {
  T::try_from(value).map_err(|_| XrfError::new_invalid_error(format!("{what} exceeds the {} format limit", T::LABEL)))
}

#[cfg(test)]
mod tests {
  use xrf_error::XrfResult;

  use crate::capacity_utils::{assert_count_fits, new_bounded_vec, to_format_size};

  #[test]
  fn accepts_count_that_fits_the_budget() -> XrfResult {
    assert_count_fits(4, 32, 8, "test records")?;
    assert_count_fits(4, 33, 8, "test records")?;
    assert_count_fits(0, 0, 8, "test records")?;

    Ok(())
  }

  #[test]
  fn rejects_count_larger_than_the_budget_allows() {
    let error: String = assert_count_fits(5, 32, 8, "test records")
      .expect_err("expect the declared count to exceed the budget")
      .to_string();

    assert!(
      error.contains("test records declares 5 entries"),
      "Unexpected error: {error}"
    );
    assert!(error.contains("at least 8 each"), "Unexpected error: {error}");
    assert!(error.contains("only 32 remain"), "Unexpected error: {error}");
  }

  #[test]
  fn rejects_impossible_count_without_reserving_it() {
    let error: String = new_bounded_vec::<u64>(u64::from(u32::MAX), 4, 8, "test records")
      .expect_err("expect the declared count to exceed the budget")
      .to_string();

    assert!(
      error.contains("declares 4294967295 entries"),
      "Unexpected error: {error}"
    );
  }

  #[test]
  fn reserves_exactly_the_declared_count() -> XrfResult {
    let vector: Vec<u64> = new_bounded_vec(4, 32, 8, "test records")?;

    assert!(vector.is_empty(), "Expect an empty vector");
    assert_eq!(vector.capacity(), 4, "Expect capacity for the declared count");

    Ok(())
  }

  #[test]
  fn treats_a_zero_minimum_as_one() {
    let error: String = assert_count_fits(9, 8, 0, "test records")
      .expect_err("expect the declared count to exceed the budget")
      .to_string();

    assert!(error.contains("only 8 remain"), "Unexpected error: {error}");
  }

  #[test]
  fn narrows_a_length_that_fits_its_format_field() -> XrfResult {
    assert_eq!(to_format_size::<u8>(255, "test records")?, 255u8);
    assert_eq!(to_format_size::<u16>(65535, "test records")?, 65535u16);
    assert_eq!(to_format_size::<u32>(u32::MAX as usize, "test records")?, u32::MAX);

    Ok(())
  }

  #[test]
  fn rejects_a_length_past_its_format_field() {
    assert_eq!(
      to_format_size::<u8>(256, "test records")
        .expect_err("expect the length to exceed its format field")
        .to_string(),
      "Invalid error: test records exceeds the u8 format limit"
    );

    assert_eq!(
      to_format_size::<u16>(65536, "test records")
        .expect_err("expect the length to exceed its format field")
        .to_string(),
      "Invalid error: test records exceeds the u16 format limit"
    );

    // The u32 boundary is reachable here and nowhere else: the helper takes a length rather than a collection.
    assert_eq!(
      to_format_size::<u32>(u32::MAX as usize + 1, "test records")
        .expect_err("expect the length to exceed its format field")
        .to_string(),
      "Invalid error: test records exceeds the u32 format limit"
    );
  }
}
