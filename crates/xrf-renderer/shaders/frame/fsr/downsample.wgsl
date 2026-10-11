#import "common/fullscreen"
#import "frame/fsr/common"

// The single-pass downsampler's step (`ffx_spd.h`, `SpdReduce4` as both FSR pyramids define it), one level a pass:
// each texel the mean of the two by two texels under it. It makes the farthest depth at half size, the one output of
// `ffx_fsr3upscaler_luma_pyramid.h` read with an exposure of one (`StoreFarthestDepthMip1`), and the shading change
// pyramid's levels after its first.

@group(0) @binding(1) var source: texture_2d<f32>;

@fragment
fn fs_downsample(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let size: vec2<f32> = vec2<f32>(textureDimensions(source));
  let base: vec2<f32> = floor(in.clip.xy) * 2.0;
  var sum: vec4<f32> = vec4<f32>(0.0);

  for (var corner: i32 = 0; corner < 4; corner++) {
    sum += textureLoad(source, clamped_texel(base + footprint_corner(corner), size), 0);
  }

  return sum * 0.25;
}
