use xrf_error_derive::ErrorConstructors;

#[derive(ErrorConstructors)]
enum ExampleError {
  #[constructor]
  #[constructor("custom_error")]
  Invalid { message: String },
}

fn main() {}
