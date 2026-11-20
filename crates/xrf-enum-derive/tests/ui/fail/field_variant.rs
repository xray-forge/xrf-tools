use xrf_enum_derive::EnumVariants;

#[derive(EnumVariants)]
enum Shape {
  Point,
  Circle(f32),
  Box { width: f32 },
}

fn main() {}
