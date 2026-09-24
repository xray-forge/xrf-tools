import {
  attribute,
  cameraPosition,
  cameraViewMatrix,
  Discard,
  float,
  Fn,
  If,
  mix,
  normalize,
  select,
  uint,
  uniform,
  varying,
  vec3,
  vec4,
} from "three/tsl";
import { Node, TextureNode } from "three/webgpu";

import { IRendererSurface } from "#/contract/scene/renderer-surface";
import { MaterialSamplers } from "#/material/material-samplers";
import { ISurfaceShader } from "#/material/surface-shader";
import { DEFAULT_GLOSS, MATERIAL_SLICES } from "#/material/surface-texel.tsl";
import { toGBufferOutput } from "#/shader/gbuffer.tsl";
import { toListedImpostor } from "#/shader/placement.tsl";
import { getWhiteTexture } from "#/texture/placeholder-textures";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";
import { STATIC_LOD_CORNER_COLUMNS } from "#/uniforms/static-draw-buffers";

/** `clip(D.w - 96.h/255.h)`: what an impostor's faded alpha is cut at (`lod.ps`). */
const IMPOSTOR_ALPHA_REFERENCE: number = 96 / 255;

/** `L_SCALE`, what `lod.vs` scales a corner's hemisphere term by. */
const HEMI_SCALE: number = 2 * 1.55;

/** The lighting model `lod.ps` writes: `pack_gbuffer`'s position `w` of zero. */
const IMPOSTOR_MATERIAL: number = 0;

/**
 * The companion's normal, baked in the engine's view space, where `+z` looks away from the eye, turned into this
 * renderer's, where it looks towards it. `lod.ps` writes it to the G-buffer as the view normal it is.
 */
function toViewNormal(companion: Node<"vec3">): Node<"vec3"> {
  const normal: Node<"vec3"> = normalize(companion.mul(2).sub(1));

  return vec3(normal.x, normal.y, normal.z.negate());
}

/** The first of the two columns a corner of an impostor's facet takes. */
function toCornerColumn(lod: Node<"uint">, facet: Node<"uint">, vertex: Node<"uint">): Node<"uint"> {
  return lod.mul(STATIC_LOD_CORNER_COLUMNS).add(facet.mul(4).add(vertex).mul(2));
}

/**
 * `details\lod` (`lod.vs`, `lod.ps`, `render_lods`): an impostor as a quad of the two facets facing the camera best,
 * each corner blended between them by the LOD cull's factor and pulled half the sphere's radius towards the camera.
 * The atlas is sampled at both facets' coordinates and blended the same way, its alpha faded by the cull and cut at
 * 96; the `_nm` companion gives the view normal and the hemisphere term, times the corners'.
 *
 * @param surface - The impostor surface.
 * @param samplers - Where its slots are bound.
 * @param uniforms - What the frame's shaders read.
 * @returns Its shader, placing its own vertices.
 */
export function toImpostorSurfaceShader(
  surface: IRendererSurface,
  samplers: MaterialSamplers,
  uniforms: RendererUniforms
): ISurfaceShader {
  const buffers = uniforms.staticDraws;
  const lod: Node<"uint"> = toListedImpostor(buffers);
  const terms = buffers.lodTermColumns.element(lod) as unknown as Node<"uvec4">;
  const factor: Node<"float"> = float(terms.z.shiftRight(8).bitAnd(255)).div(255);
  const alpha: Node<"float"> = float(terms.z.bitAnd(255)).div(255);
  // A quad's corners are written in the order `render_lods` takes them from a facet: 3, 0, 2, 1.
  const corner: Node<"uint"> = attribute<"vec2">("uv", "vec2").x.toUint();
  const vertex: Node<"uint"> = select(
    corner.equal(0),
    uint(3),
    select(corner.equal(1), uint(0), select(corner.equal(2), uint(2), uint(1)))
  );
  const nextColumn: Node<"uint"> = toCornerColumn(lod, terms.y, vertex);
  const bestColumn: Node<"uint"> = toCornerColumn(lod, terms.x, vertex);
  const next = buffers.lodCornerColumns.element(nextColumn) as unknown as Node<"vec4">;
  const best = buffers.lodCornerColumns.element(bestColumn) as unknown as Node<"vec4">;
  const nextAtlas = buffers.lodCornerColumns.element(nextColumn.add(1)) as unknown as Node<"vec4">;
  const bestAtlas = buffers.lodCornerColumns.element(bestColumn.add(1)) as unknown as Node<"vec4">;
  const sphere = buffers.lodSphereColumns.element(lod) as unknown as Node<"vec4">;
  const shift: Node<"vec3"> = normalize(sphere.xyz.sub(cameraPosition)).mul(sphere.w.mul(-0.5));
  const position: Node<"vec3"> = mix(next.xyz, best.xyz, factor).add(shift);

  const blend: Node<"float"> = varying(factor);
  const fade: Node<"float"> = varying(alpha);
  const hemi: Node<"float"> = varying(mix(next.w, best.w, factor).mul(HEMI_SCALE));
  const base0: TextureNode = samplers.bind(surface.textures.base, getWhiteTexture(), varying(nextAtlas.xy));
  const base1: TextureNode = samplers.bind(surface.textures.base, getWhiteTexture(), varying(bestAtlas.xy));
  const normal0: TextureNode = samplers.bind(surface.textures.hemi, getWhiteTexture(), varying(nextAtlas.xy));
  const normal1: TextureNode = samplers.bind(surface.textures.hemi, getWhiteTexture(), varying(bestAtlas.xy));

  const companion: Node<"vec4"> = mix(normal0, normal1, blend);
  // The discard rides on the albedo, as a cut-out's does: the output struct cannot be what a function returns.
  const albedo: Node<"vec4"> = Fn(() => {
    const color: Node<"vec4"> = mix(base0, base1, blend);

    If(color.w.mul(fade).lessThan(IMPOSTOR_ALPHA_REFERENCE), () => {
      Discard();
    });

    return vec4(color.xyz, DEFAULT_GLOSS);
  })();

  return {
    fragmentNode: toGBufferOutput(
      albedo,
      toViewNormal(companion.xyz),
      companion.w.mul(hemi),
      // Unoccluded, as a tree writes it: the engine lights both with the sun's shadow map, which this renderer has not.
      float(1),
      uniform((IMPOSTOR_MATERIAL + 0.5) / MATERIAL_SLICES)
    ),
    positionViewNode: cameraViewMatrix.mul(vec4(position, 1)).xyz,
  };
}
