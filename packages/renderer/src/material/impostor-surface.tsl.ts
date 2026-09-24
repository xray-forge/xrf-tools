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

/** The first of the two columns a corner of an impostor's facet takes. */
function toCornerColumn(lod: Node<"uint">, facet: Node<"uint">, vertex: Node<"uint">): Node<"uint"> {
  return lod.mul(STATIC_LOD_CORNER_COLUMNS).add(facet.mul(4).add(vertex).mul(2));
}

/**
 * `details\lod` (`lod.vs`, `lod.ps`, `render_lods`): an impostor as a quad of the two facets facing the camera best,
 * each corner blended between them by the LOD cull's factor and pulled half the sphere's radius towards the camera.
 * The atlas is sampled at both facets' coordinates and blended the same way, its alpha faded by the cull and cut at
 * 96; the `_nm` companion gives the normal, written as it is, and the hemisphere term, times the corners'. The
 * corners' baked sun stands in for the shadow map the engine lights them with.
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
  // Standing in for the sun's shadow map, as a tree's own baked sun does: the corners' sun, blended like the hemi.
  const sun: Node<"float"> = varying(mix(nextAtlas.z, bestAtlas.z, factor));
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
      normalize(companion.xyz.mul(2).sub(1)),
      companion.w.mul(hemi),
      sun,
      uniform((IMPOSTOR_MATERIAL + 0.5) / MATERIAL_SLICES)
    ),
    positionViewNode: cameraViewMatrix.mul(vec4(position, 1)).xyz,
  };
}
