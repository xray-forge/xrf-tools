use xrf_enum_derive::EnumDisplay;

#[derive(EnumDisplay)]
enum Buffer<const N: usize> {
  Empty,
}

fn main() {}
