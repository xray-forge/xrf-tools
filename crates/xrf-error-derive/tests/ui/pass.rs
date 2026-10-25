use xrf_error_derive::ErrorConstructors;

#[derive(Debug, ErrorConstructors, PartialEq)]
enum ExampleError {
  #[constructor]
  Automatic { message: String },
  #[constructor("custom_error")]
  Custom { message: String },
  Ignored,
}

#[derive(Debug, ErrorConstructors, PartialEq)]
enum GenericError<T: Clone> {
  #[constructor]
  Invalid { message: String },
  Holding(T),
}

fn main() {
  assert_eq!(
    GenericError::<u8>::new_invalid_error("generic"),
    GenericError::Invalid {
      message: String::from("generic"),
    }
  );
  let _ = GenericError::Holding(0_u8);

  assert_eq!(
    ExampleError::new_automatic_error("automatic"),
    ExampleError::Automatic {
      message: String::from("automatic"),
    }
  );
  assert_eq!(
    ExampleError::custom_error(String::from("custom")),
    ExampleError::Custom {
      message: String::from("custom"),
    }
  );
  let _ = ExampleError::Ignored;
}
