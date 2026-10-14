enable wgpu_binding_array;

#import "common/camera"
#import "grass/records"
#import "generated/grass/card_shadow"

// The grass's cards drawn into a sun cascade's map, where the cascades shadow the frame: the triangles
// `grass/cards.wgsl` stood this frame, each corner read by its index, depth alone, cut out by the model's texture as
// the grass draw cuts it, so light falls between the blades. A triangle the frame left unwritten has corners that are
// not a number, which is drawn as none. It reads the cards' corners, three floats each, their triangles' records (model
// and texture widths a metre, then each corner's coordinate) and the models at group 2.

// The texture cache's group, at 1 as every static draw binds it.
@group(1) @binding(0) var textures: binding_array<texture_2d<f32>>;
@group(1) @binding(1) var texture_sampler: sampler;

// Words a triangle's corners and record take, as `grass/cards.wgsl` writes them.
const CARD_VERTEX_WORDS: u32 = 9u;
const CARD_TRIANGLE_WORDS: u32 = 4u;

struct CardVarying {
  @builtin(position) clip: vec4<f32>,
  @location(0) uv: vec2<f32>,
  @location(1) @interpolate(flat) model: u32,
};

@vertex
fn vs_card_shadow(@builtin(vertex_index) index: u32) -> CardVarying {
  let triangle: u32 = index / 3u;
  let corner: u32 = index % 3u;
  let at: u32 = triangle * CARD_VERTEX_WORDS + corner * 3u;
  let position: vec3<f32> = vec3<f32>(bitcast<f32>(card_vertices[at]), bitcast<f32>(card_vertices[at + 1u]),
    bitcast<f32>(card_vertices[at + 2u]));
  let record: u32 = triangle * CARD_TRIANGLE_WORDS;
  var out: CardVarying;

  out.uv = unpack2x16float(card_triangles[record + 1u + corner]);
  out.model = card_triangles[record] & 0xffffu;
  // A corner that is not a number stands for no triangle, told by its bits: a compiler may take a float to be a number.
  let is_none: bool = (card_vertices[at] & 0x7fffffffu) > 0x7f800000u;

  out.clip = select(camera.view_projection * vec4<f32>(position, 1.0), vec4<f32>(0.0, 0.0, -1.0, 1.0), is_none);

  return out;
}

// The finest the cut is read at past the texture's own: a cascade's texel spans many of the texture's, and read at the
// level that would match it, the alpha blurred and raised, cut out as blobs two or three times the tuft that shrank as
// the tuft came into a finer cascade. Read fine and cut as it stands, a texel is shadowed where a blade crosses it,
// which the sun pass's filter softens.
const CARD_SHADOW_LEVEL_MOST: f32 = 1.0;

@fragment
fn fs_card_shadow(in: CardVarying) {
  let model: GrassModel = models[in.model];
  let size: vec2<f32> = vec2<f32>(textureDimensions(textures[model.texture]));
  let dx: vec2<f32> = dpdx(in.uv) * size;
  let dy: vec2<f32> = dpdy(in.uv) * size;
  let level: f32 = min(0.5 * log2(max(max(dot(dx, dx), dot(dy, dy)), 1e-8)), CARD_SHADOW_LEVEL_MOST);
  let base: vec4<f32> = textureSampleLevel(textures[model.texture], texture_sampler, in.uv, level);

  if (base.a < model.alpha_reference) {
    discard;
  }
}
