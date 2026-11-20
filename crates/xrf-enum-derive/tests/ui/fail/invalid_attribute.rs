use xrf_enum_derive::EnumDisplay;

#[derive(EnumDisplay)]
enum Language {
  #[display(eng)]
  English,
}

fn main() {}
