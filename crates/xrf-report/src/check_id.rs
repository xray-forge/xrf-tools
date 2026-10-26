use crate::identifier::identifier;

identifier!(CheckId, "check ID");

#[cfg(test)]
mod tests {
  use crate::CheckId;

  #[test]
  fn rejects_an_empty_identifier() {
    assert_eq!(CheckId::new("").unwrap_err().to_string(), "check ID cannot be empty");
    assert_eq!(
      CheckId::new("textures").map(|it| it.to_string()),
      Ok(String::from("textures"))
    );
  }
}
