use crate::identifier::identifier;

identifier!(RuleId, "rule ID");

#[cfg(test)]
mod tests {
  use crate::RuleId;

  #[test]
  fn rejects_an_empty_identifier() {
    assert_eq!(RuleId::new("").unwrap_err().to_string(), "rule ID cannot be empty");
    assert_eq!(RuleId::of_static("textures.read").as_str(), "textures.read");
  }
}
