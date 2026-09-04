use xrf_error::XrfResult;

use super::source_span::SourceSpan;

/// One comma-separated condition-list branch.
#[derive(Clone, Debug, PartialEq)]
pub struct CondlistBranch {
  pub conditions: Vec<CondlistCondition>,
  pub effects: Vec<CondlistCondition>,
  pub result: Option<String>,
  pub span: SourceSpan,
}

/// A condition or effect in a condition-list branch.
#[derive(Clone, Debug, PartialEq)]
pub enum CondlistCondition {
  InfoPortion {
    name: String,
    required: bool,
    span: SourceSpan,
  },
  Probability {
    value: f64,
    span: SourceSpan,
  },
  Function {
    expected: bool,
    name: String,
    parameters: Option<Vec<String>>,
    span: SourceSpan,
  },
}

impl CondlistBranch {
  pub fn parse(branch: &str, branch_offset: usize) -> XrfResult<CondlistBranch> {
    let (conditions, conditions_span): (Vec<CondlistCondition>, Option<SourceSpan>) =
      Self::parse_delimited_conditions(branch, branch_offset, b'{', b'}', "condition")?;
    let (effects, effects_span): (Vec<CondlistCondition>, Option<SourceSpan>) =
      Self::parse_delimited_conditions(branch, branch_offset, b'%', b'%', "effect")?;
    let result: Option<String> = Self::parse_result(branch, conditions_span, effects_span)?;

    if result.is_none() && conditions.is_empty() && effects.is_empty() {
      return Err(SourceSpan::parsing_error(
        branch_offset,
        "Expected a result, condition list, or effect list",
      ));
    }

    Ok(CondlistBranch {
      conditions,
      effects,
      result,
      span: SourceSpan::new(branch_offset, branch_offset + branch.len()),
    })
  }

  fn parse_delimited_conditions(
    value: &str,
    value_offset: usize,
    opening: u8,
    closing: u8,
    name: &str,
  ) -> XrfResult<(Vec<CondlistCondition>, Option<SourceSpan>)> {
    let opening_index: Option<usize> = Self::find_delimiter(value, 0, opening);
    let closing_index: Option<usize> = Self::find_delimiter(value, 0, closing);

    let Some(opening_index) = opening_index else {
      if closing != opening
        && let Some(closing_index) = closing_index
      {
        return Err(SourceSpan::parsing_error(
          value_offset + closing_index,
          &format!("Unexpected closing '{}' for the {name} list", closing as char),
        ));
      }

      return Ok((Vec::new(), None));
    };

    if closing != opening && closing_index.is_some_and(|index| index < opening_index) {
      return Err(SourceSpan::parsing_error(
        value_offset + closing_index.expect("Closing delimiter should be present"),
        &format!("Unexpected closing '{}' for the {name} list", closing as char),
      ));
    }

    let closing_index: usize = Self::find_delimiter(value, opening_index + 1, closing).ok_or_else(|| {
      SourceSpan::parsing_error(
        value_offset + opening_index,
        &format!("Expected closing '{}' for the {name} list", closing as char),
      )
    })?;

    if opening != closing
      && Self::find_delimiter(value, opening_index + 1, opening).is_some_and(|index| index < closing_index)
    {
      return Err(SourceSpan::parsing_error(
        value_offset + opening_index + 1,
        &format!("Unexpected opening '{}' for the {name} list", opening as char),
      ));
    }

    if let Some(index) = Self::find_delimiter(value, closing_index + 1, opening) {
      return Err(SourceSpan::parsing_error(
        value_offset + index,
        &format!("Unexpected '{}' after the {name} list", opening as char),
      ));
    }

    if closing != opening
      && let Some(index) = Self::find_delimiter(value, closing_index + 1, closing)
    {
      return Err(SourceSpan::parsing_error(
        value_offset + index,
        &format!("Unexpected '{}' after the {name} list", closing as char),
      ));
    }

    let span: SourceSpan = SourceSpan::new(opening_index, closing_index + 1);
    let conditions: Vec<CondlistCondition> = Self::parse_conditions(
      &value[opening_index + 1..closing_index],
      value_offset + opening_index + 1,
    )?;

    Ok((conditions, Some(span)))
  }

  fn parse_result(
    value: &str,
    conditions_span: Option<SourceSpan>,
    effects_span: Option<SourceSpan>,
  ) -> XrfResult<Option<String>> {
    let mut spans: Vec<SourceSpan> = [conditions_span, effects_span].into_iter().flatten().collect();
    spans.sort_by_key(|span| span.start);

    let mut result: String = String::new();
    let mut cursor: usize = 0;

    for span in spans {
      result.push_str(&value[cursor..span.start]);
      cursor = span.end;
    }

    result.push_str(&value[cursor..]);

    let result: &str = result.trim();

    if result.is_empty() {
      return Ok(None);
    }

    Ok(Some(String::from(result)))
  }

  fn parse_conditions(value: &str, value_offset: usize) -> XrfResult<Vec<CondlistCondition>> {
    let mut conditions: Vec<CondlistCondition> = Vec::new();
    let mut cursor: usize = 0;

    while cursor < value.len() {
      Self::skip_whitespace(value, &mut cursor);

      if cursor == value.len() {
        break;
      }

      conditions.push(Self::parse_condition(value, value_offset, &mut cursor)?);
    }

    Ok(conditions)
  }

  fn parse_condition(value: &str, value_offset: usize, cursor: &mut usize) -> XrfResult<CondlistCondition> {
    let token_start: usize = *cursor;
    let sign: u8 = Self::byte_at(value, *cursor).expect("Cursor should point to a condition token");

    if !Self::is_condition_sign(sign) {
      return Err(SourceSpan::parsing_error(
        value_offset + *cursor,
        "Expected a condition or effect prefix ('+', '-', '~', '=', or '!')",
      ));
    }

    *cursor += 1;

    if matches!(sign, b'=' | b'!') {
      Self::skip_whitespace(value, cursor);
    }

    let name_start: usize = *cursor;
    let mut name_end: Option<usize> = None;
    let mut has_function_call: bool = false;
    let mut parameters: Option<Vec<String>> = None;

    while let Some(byte) = Self::byte_at(value, *cursor) {
      if byte.is_ascii_whitespace() {
        if matches!(sign, b'=' | b'!') {
          let mut function_start: usize = *cursor;
          Self::skip_whitespace(value, &mut function_start);

          if Self::byte_at(value, function_start) == Some(b'(') {
            name_end = Some(*cursor);
            has_function_call = true;
            let (next_cursor, parsed_parameters): (usize, Vec<String>) =
              Self::parse_function_call(value, function_start, value_offset)?;
            *cursor = next_cursor;
            parameters = Some(parsed_parameters);

            if let Some(next) = Self::byte_at(value, *cursor)
              && !next.is_ascii_whitespace()
              && !Self::is_condition_sign(next)
            {
              return Err(SourceSpan::parsing_error(
                value_offset + *cursor,
                "Unexpected data after function call",
              ));
            }

            continue;
          }
        }

        break;
      }

      if Self::is_condition_sign(byte) {
        break;
      }

      if matches!(byte, b',' | b'{' | b'}' | b'%') {
        return Err(SourceSpan::parsing_error(
          value_offset + *cursor,
          "Unexpected delimiter in a condition or effect",
        ));
      }

      if byte == b')' {
        return Err(SourceSpan::parsing_error(
          value_offset + *cursor,
          "Unexpected ')' in a condition or effect",
        ));
      }

      if byte == b'(' {
        if *cursor == name_start || has_function_call {
          return Err(SourceSpan::parsing_error(
            value_offset + *cursor,
            "Expected one function call after a condition or effect name",
          ));
        }

        if !matches!(sign, b'=' | b'!') {
          return Err(SourceSpan::parsing_error(
            value_offset + token_start,
            "Only '=' and '!' tokens can call functions",
          ));
        }

        name_end = Some(*cursor);
        has_function_call = true;

        let (next_cursor, parsed_parameters): (usize, Vec<String>) =
          Self::parse_function_call(value, *cursor, value_offset)?;

        *cursor = next_cursor;
        parameters = Some(parsed_parameters);

        if let Some(next) = Self::byte_at(value, *cursor)
          && !next.is_ascii_whitespace()
          && !Self::is_condition_sign(next)
        {
          return Err(SourceSpan::parsing_error(
            value_offset + *cursor,
            "Unexpected data after function call",
          ));
        }

        continue;
      }

      *cursor += 1;
    }

    if name_start == *cursor {
      return Err(SourceSpan::parsing_error(
        value_offset + token_start,
        "Expected a name after condition or effect prefix",
      ));
    }

    let name: String = value[name_start..name_end.unwrap_or(*cursor)].to_owned();
    let span: SourceSpan = SourceSpan::new(value_offset + token_start, value_offset + *cursor);

    match sign {
      b'+' => Ok(CondlistCondition::InfoPortion {
        name,
        required: true,
        span,
      }),
      b'-' => Ok(CondlistCondition::InfoPortion {
        name,
        required: false,
        span,
      }),
      b'~' => match name.parse::<f64>() {
        Ok(value) => Ok(CondlistCondition::Probability { value, span }),
        Err(_) => Err(SourceSpan::parsing_error(
          value_offset + name_start,
          "Expected a numeric probability after '~'",
        )),
      },
      b'=' => Ok(CondlistCondition::Function {
        expected: true,
        name,
        parameters,
        span,
      }),
      b'!' => Ok(CondlistCondition::Function {
        expected: false,
        name,
        parameters,
        span,
      }),
      _ => unreachable!("Condition signs are checked above"),
    }
  }
  fn parse_function_call(value: &str, open_parenthesis: usize, value_offset: usize) -> XrfResult<(usize, Vec<String>)> {
    let mut cursor: usize = open_parenthesis + 1;

    while let Some(byte) = Self::byte_at(value, cursor) {
      if byte == b')' {
        let parameters_raw: &str = &value[open_parenthesis + 1..cursor];
        let parameters: Vec<String> = if parameters_raw.trim().is_empty() {
          Vec::new()
        } else {
          parameters_raw
            .split(':')
            .map(|parameter| parameter.trim().to_owned())
            .collect()
        };

        return Ok((cursor + 1, parameters));
      }

      if matches!(byte, b'(' | b',' | b'{' | b'}' | b'%') {
        return Err(SourceSpan::parsing_error(
          value_offset + cursor,
          "Unexpected character in function parameters",
        ));
      }

      cursor += 1;
    }

    Err(SourceSpan::parsing_error(
      value_offset + open_parenthesis,
      "Expected closing ')' for function call",
    ))
  }

  fn find_delimiter(value: &str, start: usize, delimiter: u8) -> Option<usize> {
    value.as_bytes()[start..]
      .iter()
      .position(|byte| *byte == delimiter)
      .map(|position| start + position)
  }

  fn skip_whitespace(value: &str, cursor: &mut usize) {
    while Self::byte_at(value, *cursor).is_some_and(|byte| byte.is_ascii_whitespace()) {
      *cursor += 1;
    }
  }

  fn byte_at(value: &str, index: usize) -> Option<u8> {
    value.as_bytes().get(index).copied()
  }

  fn is_condition_sign(value: u8) -> bool {
    matches!(value, b'+' | b'-' | b'~' | b'=' | b'!')
  }
}
