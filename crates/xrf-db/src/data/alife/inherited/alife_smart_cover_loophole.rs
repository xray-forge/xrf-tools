use serde::{Deserialize, Serialize};
use xrf_error::{XrfError, XrfResult};

#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeSmartCoverLoophole {
  pub name: String,
  pub enabled: u8,
}

impl AlifeSmartCoverLoophole {
  /// Serialize list of loopholes into single string.
  pub fn list_to_string(loopholes: &[Self]) -> String {
    loopholes
      .iter()
      .map(|loophole| format!("{}:{}", loophole.name, loophole.enabled))
      .collect::<Vec<_>>()
      .join(",")
  }

  /// Read list of loopholes from string.
  ///
  /// A smart cover may carry no loopholes at all, which `list_to_string` renders as an empty value.
  /// Splitting that would yield one empty entry rather than none.
  pub fn string_to_list(value: &str) -> XrfResult<Vec<Self>> {
    let mut loopholes: Vec<Self> = Vec::new();

    if value.trim().is_empty() {
      return Ok(loopholes);
    }

    for it in value.split(',').map(|it| it.trim()) {
      let partial: Vec<&str> = it.split(':').map(|it| it.trim()).collect::<Vec<&str>>();

      if partial.len() == 2 {
        loopholes.push(Self {
          name: String::from(*partial.first().unwrap()),
          enabled: match partial.last().unwrap().parse::<u8>() {
            Ok(parsed) => parsed,
            Err(_) => {
              return Err(XrfError::new_parsing_error("Failed to parse loophole enabled status"));
            }
          },
        })
      } else {
        return Err(XrfError::new_parsing_error(
          "Invalid value provided for loopholes parsing, ':' separated values expected",
        ));
      }
    }

    Ok(loopholes)
  }
}

#[cfg(test)]
mod tests {
  use xrf_error::XrfResult;

  use crate::data::alife::inherited::alife_smart_cover_loophole::AlifeSmartCoverLoophole;

  #[test]
  fn test_list_to_string_round_trip() -> XrfResult {
    let original: Vec<AlifeSmartCoverLoophole> = vec![
      AlifeSmartCoverLoophole {
        name: String::from("first"),
        enabled: 1,
      },
      AlifeSmartCoverLoophole {
        name: String::from("second"),
        enabled: 0,
      },
    ];

    let serialized: String = AlifeSmartCoverLoophole::list_to_string(&original);

    assert_eq!(serialized, "first:1,second:0");
    assert_eq!(AlifeSmartCoverLoophole::string_to_list(&serialized)?, original);

    Ok(())
  }

  /// Smart covers with no loopholes exist in the Call of Chernobyl and Anomaly spawns, and render as
  /// an empty value that must read back as no loopholes rather than as one malformed entry.
  #[test]
  fn test_empty_list_round_trip() -> XrfResult {
    let serialized: String = AlifeSmartCoverLoophole::list_to_string(&[]);

    assert_eq!(serialized, "");
    assert_eq!(AlifeSmartCoverLoophole::string_to_list(&serialized)?, vec![]);
    assert_eq!(AlifeSmartCoverLoophole::string_to_list("   ")?, vec![]);

    Ok(())
  }

  #[test]
  fn test_malformed_entry_is_rejected() {
    assert!(AlifeSmartCoverLoophole::string_to_list("first").is_err());
    assert!(AlifeSmartCoverLoophole::string_to_list("first:not-a-number").is_err());
  }
}
