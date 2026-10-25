use xrf_error_derive::ErrorConstructors;

#[derive(ErrorConstructors)]
enum ExampleError {
  #[constructor]
  Invalid { message: String, line: usize },
}

fn main() {}
