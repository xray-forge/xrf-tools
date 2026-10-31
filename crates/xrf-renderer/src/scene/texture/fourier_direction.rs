/// Which way a discrete Fourier transform runs: from values to frequencies, or back, scaled by the count.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FourierDirection {
  Forward,
  Inverse,
}
