/// What a weather texture is sampled as.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum WeatherTextureKind {
  /// A sky or its irradiance, six faces sampled by direction.
  Cube,
  /// One picture: the clouds tiled over the dome, a rain streak, the flow down a wall.
  Flat,
  /// A volume's slices as layers, which a shader blends between: the splashes rippling on wet ground.
  Volume,
}
