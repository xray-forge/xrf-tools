use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub};
use std::str::FromStr;

use byteorder::ByteOrder;
use derive_more::Display;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkFieldReadWrite, ChunkFieldReader, ChunkFieldWriter};
use xrf_error::{XrfError, XrfResult};

#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Display)]
#[serde(rename_all = "camelCase")]
#[display("{x},{y},{z}")]
pub struct Vector3d<T = f32> {
  pub x: T,
  pub y: T,
  pub z: T,
}

impl Vector3d<f32> {
  pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);
  pub const X: Self = Self::new(1.0, 0.0, 0.0);
  pub const Y: Self = Self::new(0.0, 1.0, 0.0);
  pub const Z: Self = Self::new(0.0, 0.0, 1.0);
  pub const ONE: Self = Self::new(1.0, 1.0, 1.0);

  pub const fn new(x: f32, y: f32, z: f32) -> Self {
    Self { x, y, z }
  }

  pub fn dot(&self, other: &Self) -> f32 {
    self.x * other.x + self.y * other.y + self.z * other.z
  }

  pub fn cross(&self, other: &Self) -> Self {
    Self::new(
      self.y * other.z - self.z * other.y,
      self.z * other.x - self.x * other.z,
      self.x * other.y - self.y * other.x,
    )
  }

  pub fn length_squared(&self) -> f32 {
    self.dot(self)
  }

  pub fn length(&self) -> f32 {
    self.length_squared().sqrt()
  }

  pub fn distance_squared(&self, other: &Self) -> f32 {
    (self - other).length_squared()
  }

  pub fn distance(&self, other: &Self) -> f32 {
    self.distance_squared(other).sqrt()
  }

  /// The same direction at unit length, or zero for a vector with none.
  pub fn normalize_or_zero(&self) -> Self {
    let length: f32 = self.length();

    if length > 0.0 { self / length } else { Self::ZERO }
  }

  /// The smaller of each pair of components, which is how a box's lower corner grows to take in a point.
  pub fn min(&self, other: &Self) -> Self {
    Self::new(self.x.min(other.x), self.y.min(other.y), self.z.min(other.z))
  }

  /// The larger of each pair of components, which is how a box's upper corner grows to take in a point.
  pub fn max(&self, other: &Self) -> Self {
    Self::new(self.x.max(other.x), self.y.max(other.y), self.z.max(other.z))
  }

  /// The point halfway to `other`.
  pub fn midpoint(&self, other: &Self) -> Self {
    &(self + other) * 0.5
  }

  pub fn to_array(self) -> [f32; 3] {
    [self.x, self.y, self.z]
  }

  /// The unit direction a 16-bit compressed normal names, `pvDecompress` (`xrCore/_compressed_normal.cpp`).
  ///
  /// The top three bits are the signs of x, y and z; the remaining thirteen are a point on the triangle
  /// (0,0)-(0,126)-(126,0) folded into a rectangle, six bits of x above seven of y. A level AI node packs its plane
  /// normal this way.
  ///
  /// As SDK 0.7's xrCore computes it, which every plane value's round trip through its xrAI confirms: the lookup
  /// table's `1 / sqrt` taken on the x87 at extended precision before the engine lowers it, stored as a float; each
  /// component that float times its bits, in single precision; and a sign bit forced on, not a negation, so the
  /// folded edge's z of -1 stays negative under its sign bit.
  pub fn from_compressed_normal(packed: u16) -> Self {
    let mut x: i32 = i32::from((packed & 0x1F80) >> 7);
    let mut y: i32 = i32::from(packed & 0x007F);

    if x + y >= 127 {
      x = 127 - x;
      y = 127 - y;
    }

    let z: i32 = 126 - x - y;
    let (xd, yd, zd): (f64, f64, f64) = (f64::from(x), f64::from(y), f64::from(z));
    let adjustment: f32 = (1.0 / (zd * zd + yd * yd + xd * xd).sqrt()) as f32;
    let component = |bits: i32, sign: u16| {
      let value: f32 = bits as f32 * adjustment;

      if packed & sign == 0 { value } else { -value.abs() }
    };

    Self::new(component(x, 0x8000), component(y, 0x4000), component(z, 0x2000))
  }

  /// The 16-bit compressed normal of this direction, `pvCompress` (`xrCore/_compressed_normal.cpp:43`): the signs in
  /// the top three bits, then the direction projected onto the plane through the three axes' unit points, its x and y
  /// floored to 126ths and folded into the rectangle [`Self::from_compressed_normal`] reads. Need not be unit length.
  ///
  /// In single precision as SDK 0.7's xrCore computes it, its sum taken z first, `(z + y) + x`: the order decides
  /// which side of a whole 126th a component lands on.
  pub fn to_compressed_normal(self) -> u16 {
    let signs: u16 = (u16::from(self.x.is_sign_negative()) << 15)
      | (u16::from(self.y.is_sign_negative()) << 14)
      | (u16::from(self.z.is_sign_negative()) << 13);
    let (x, y, z): (f32, f32, f32) = (self.x.abs(), self.y.abs(), self.z.abs());
    let scale: f32 = 126.0 / (z + y + x);
    let (mut x_bits, mut y_bits): (i32, i32) = ((x * scale).floor() as i32, (y * scale).floor() as i32);

    if x_bits >= 64 {
      x_bits = 127 - x_bits;
      y_bits = 127 - y_bits;
    }

    signs | (((x_bits << 7) | y_bits) as u16 & 0x1FFF)
  }
}

impl From<[f32; 3]> for Vector3d<f32> {
  fn from([x, y, z]: [f32; 3]) -> Self {
    Self::new(x, y, z)
  }
}

impl Add for &Vector3d<f32> {
  type Output = Vector3d<f32>;

  fn add(self, other: Self) -> Vector3d<f32> {
    Vector3d::new(self.x + other.x, self.y + other.y, self.z + other.z)
  }
}

impl AddAssign<&Self> for Vector3d<f32> {
  fn add_assign(&mut self, other: &Self) {
    self.x += other.x;
    self.y += other.y;
    self.z += other.z;
  }
}

impl Sub for &Vector3d<f32> {
  type Output = Vector3d<f32>;

  fn sub(self, other: Self) -> Vector3d<f32> {
    Vector3d::new(self.x - other.x, self.y - other.y, self.z - other.z)
  }
}

impl Mul<f32> for &Vector3d<f32> {
  type Output = Vector3d<f32>;

  fn mul(self, scale: f32) -> Vector3d<f32> {
    Vector3d::new(self.x * scale, self.y * scale, self.z * scale)
  }
}

impl Div<f32> for &Vector3d<f32> {
  type Output = Vector3d<f32>;

  fn div(self, divisor: f32) -> Vector3d<f32> {
    Vector3d::new(self.x / divisor, self.y / divisor, self.z / divisor)
  }
}

impl Neg for &Vector3d<f32> {
  type Output = Vector3d<f32>;

  fn neg(self) -> Vector3d<f32> {
    Vector3d::new(-self.x, -self.y, -self.z)
  }
}

/// An `Fvector`, `w_vec3`: three floats as one value.
impl ChunkFieldReadWrite for Vector3d<f32> {
  #[inline]
  fn read_fields<T: ByteOrder, R: ChunkFieldReader>(reader: &mut R) -> XrfResult<Self> {
    Ok(Self::from(reader.read_vector3::<T>()?))
  }

  #[inline]
  fn write_fields<T: ByteOrder, W: ChunkFieldWriter>(&self, writer: &mut W) -> XrfResult {
    writer.write_vector3::<T>(self.to_array())
  }
}

impl From<(f32, f32, f32)> for Vector3d<f32> {
  fn from(value: (f32, f32, f32)) -> Self {
    Self::new(value.0, value.1, value.2)
  }
}

impl FromStr for Vector3d<f32> {
  type Err = XrfError;

  fn from_str(string: &str) -> Result<Self, Self::Err> {
    let parts: Vec<&str> = string.split(',').collect();

    if parts.len() != 3 {
      return Err(XrfError::new_parsing_error(
        "Failed to parse 3d vector from string, expected 3 numbers",
      ));
    }

    let parse = |part: &str, axis: &str| {
      part
        .trim()
        .parse::<f32>()
        .map_err(|_| XrfError::new_parsing_error(format!("Failed to parse vector {axis} value '{}'", part.trim())))
    };

    Ok(Self {
      x: parse(parts[0], "X")?,
      y: parse(parts[1], "Y")?,
      z: parse(parts[2], "Z")?,
    })
  }
}

#[cfg(any(test, feature = "fixtures"))]
impl Vector3d {
  /// The vector this crate's own tests and its dependants' fixtures build against.
  pub fn new_mock() -> Self {
    Self {
      x: 1.1150,
      y: -0.255,
      z: 1.753,
    }
  }
}

#[cfg(test)]
mod tests {
  use std::fs::File;
  use std::io::{Seek, SeekFrom, Write, read_to_string};
  use std::str::FromStr;

  use serde_json::to_string_pretty;
  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_test_utils::{FileSlice, GeneratedResources};

  use crate::vector_3d::Vector3d;

  #[test]
  fn test_read_write() -> XrfResult {
    let filename: String = String::from("read_write.chunk");
    let mut writer: ChunkWriter = ChunkWriter::new();

    let original: Vector3d = Vector3d { x: 1.5, y: 2.7, z: 3.2 };

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 12);

    let bytes_written: usize = writer.flush_chunk_into::<XRayByteOrder>(
      &mut GeneratedResources::overwrite(&GeneratedResources::sample_name(file!(), &filename))?,
      0,
    )?;

    assert_eq!(bytes_written, 12);

    let file: FileSlice = GeneratedResources::open_slice(&GeneratedResources::sample_name(file!(), &filename))?;

    assert_eq!(file.bytes_remaining(), 12 + 8);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?
      .read_child_by_index(0)
      .expect("0 index chunk to exist");

    assert_eq!(Vector3d::read::<XRayByteOrder, _>(&mut reader)?, original);

    Ok(())
  }

  #[test]
  fn test_arithmetic() {
    let a: Vector3d = Vector3d::new(1.0, 2.0, 3.0);
    let b: Vector3d = Vector3d::new(-2.0, 0.5, 4.0);

    assert_eq!(&a + &b, Vector3d::new(-1.0, 2.5, 7.0));
    assert_eq!(&a - &b, Vector3d::new(3.0, 1.5, -1.0));
    assert_eq!(&a * 2.0, Vector3d::new(2.0, 4.0, 6.0));
    assert_eq!(&a / 2.0, Vector3d::new(0.5, 1.0, 1.5));
    assert_eq!(-&a, Vector3d::new(-1.0, -2.0, -3.0));
    assert_eq!(a.dot(&b), 11.0);
    assert_eq!(Vector3d::X.cross(&Vector3d::Y), Vector3d::new(0.0, 0.0, 1.0));
    assert_eq!(Vector3d::new(0.0, 3.0, 4.0).length(), 5.0);
    assert_eq!(
      Vector3d::new(0.0, 3.0, 4.0).normalize_or_zero(),
      Vector3d::new(0.0, 0.6, 0.8)
    );
    assert_eq!(Vector3d::ZERO.normalize_or_zero(), Vector3d::ZERO);
    assert_eq!(a.distance_squared(&Vector3d::new(1.0, 5.0, 7.0)), 25.0);
    assert_eq!(a.distance(&Vector3d::new(1.0, 5.0, 7.0)), 5.0);

    let mut sum: Vector3d = Vector3d::ZERO;

    sum += &a;
    sum += &b;

    assert_eq!(sum, &a + &b);
    assert_eq!(Vector3d::Y.cross(&Vector3d::Z), Vector3d::X);
  }

  #[test]
  fn takes_each_component_on_its_own_for_bounds() {
    let a: Vector3d = Vector3d::new(1.0, -2.0, 3.0);
    let b: Vector3d = Vector3d::new(-1.0, 5.0, 3.0);

    assert_eq!(a.min(&b), Vector3d::new(-1.0, -2.0, 3.0));
    assert_eq!(a.max(&b), Vector3d::new(1.0, 5.0, 3.0));
    assert_eq!(a.midpoint(&b), Vector3d::new(0.0, 1.5, 3.0));
  }

  #[test]
  fn decompresses_a_packed_normal_as_the_engine_does() {
    let is_near = |packed: u16, expected: Vector3d| Vector3d::from_compressed_normal(packed).distance(&expected) < 1e-6;

    // Zero bits name the triangle's (0,0) corner, which is straight along z.
    assert!(is_near(0, Vector3d::new(0.0, 0.0, 1.0)));
    // 126 in the y bits is the (0,126) corner, straight along y, and the y sign bit flips it.
    assert!(is_near(126, Vector3d::new(0.0, 1.0, 0.0)));
    assert!(is_near(0x4000 | 126, Vector3d::new(0.0, -1.0, 0.0)));
    // A point past the fold maps back: x 63 and y 127 become 64 and 0, leaving (64, 0, 62) before normalising.
    let folded: Vector3d = Vector3d::from_compressed_normal((63 << 7) | 127);

    assert!((folded.length() - 1.0).abs() < 1e-6, "Unexpected length of {folded}");
    assert!(
      (folded.x / folded.z - 64.0 / 62.0).abs() < 1e-6,
      "Unexpected normal {folded}"
    );
    assert_eq!(folded.y, 0.0);
  }

  /// On the fold's edge, x and y summing to 127, the unfolded z is -1: its sign bit forces it negative rather than
  /// flipping it positive, as the engine's `set_negative` does.
  #[test]
  fn keeps_the_folded_edge_negative_under_its_sign_bit() {
    let unsigned: Vector3d = Vector3d::from_compressed_normal(127);
    let signed: Vector3d = Vector3d::from_compressed_normal(0x2000 | 127);

    assert!(unsigned.z < 0.0, "{unsigned}");
    assert_eq!(signed, unsigned);
  }

  #[test]
  fn compresses_a_normal_as_the_engine_does() {
    assert_eq!(Vector3d::new(0.0, 0.0, 1.0).to_compressed_normal(), 0);
    assert_eq!(Vector3d::new(0.0, 1.0, 0.0).to_compressed_normal(), 126);
    assert_eq!(Vector3d::new(0.0, -1.0, 0.0).to_compressed_normal(), 0x4000 | 126);
    // Past x's 63 the point folds: (64, 0, 62) packs as x 63 and y 127.
    assert_eq!(Vector3d::new(64.0, 0.0, 62.0).to_compressed_normal(), (63 << 7) | 127);
    // A negative zero keeps its sign bit, as `negative` reads it.
    assert_eq!(Vector3d::new(-0.0, 1.0, 0.0).to_compressed_normal(), 0x8000 | 126);
  }

  #[test]
  fn test_from_to_str() -> XrfResult {
    let original: Vector3d = Vector3d {
      x: 10.5,
      y: 20.7,
      z: 30.2,
    };

    assert_eq!(original.to_string(), "10.5,20.7,30.2");
    assert_eq!(Vector3d::from_str("10.5,20.7,30.2")?, original);

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: Vector3d = Vector3d {
      x: 10.5,
      y: 20.7,
      z: 30.2,
    };

    let mut file: File =
      GeneratedResources::overwrite(&GeneratedResources::sample_name(file!(), "serialize_deserialize.json"))?;

    file.write_all(to_string_pretty(&original)?.as_bytes())?;
    file.seek(SeekFrom::Start(0))?;

    let serialized: String = read_to_string(&mut file)?;

    assert_eq!(serialized.to_string(), serialized);
    assert_eq!(original, serde_json::from_str::<Vector3d>(&serialized)?);

    Ok(())
  }
}
