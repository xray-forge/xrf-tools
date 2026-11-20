use xrf_enum_derive::EnumDisplay;

#[derive(EnumDisplay)]
enum Language {
  #[display("eng")]
  #[display("en")]
  English,
}

fn main() {}
