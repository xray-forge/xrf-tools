import {
  cameraViewMatrix,
  cos,
  Discard,
  Fn,
  fract,
  If,
  instanceIndex,
  normalize,
  positionLocal,
  select,
  sin,
  uniform,
  uv,
  varying,
  vec3,
  vec4,
} from "three/tsl";
import { Node, StorageBufferNode } from "three/webgpu";

import { IRendererSurface } from "#/contract/scene/renderer-surface";
import { MaterialSamplers } from "#/material/material-samplers";
import { ISurfaceShader } from "#/material/surface-shader";
import { DEFAULT_GLOSS, DEFAULT_MATERIAL, MATERIAL_SLICES } from "#/material/surface-texel.tsl";
import { toGBufferOutput } from "#/shader/gbuffer.tsl";
import { toPointMotion } from "#/shader/motion.tsl";
import { getWhiteTexture } from "#/texture/placeholder-textures";
import { GrassWindUniforms } from "#/uniforms/grass-wind-uniforms";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";

/** `def_aref`: where a tuft's base texture is cut out (`deffer_base_aref_flat.ps`). */
const DEFAULT_ALPHA_REFERENCE: number = 200 / 255;

/** How far below its foot a tuft's normals point from, `deffer_detail_*_flat.vs`: up, and never zero. */
const NORMAL_DROP: number = 0.75;

/** What one model's grass is drawn from. */
export interface IGrassSurfaceSource {
  surface: IRendererSurface;
  /** The model's bounding box height, which a vertex's share of the sway is measured against. */
  height: number;
  /** Every planted tuft, sorted by model: two vectors each, its place and turn, then its scale, light and wave. */
  items: StorageBufferNode<"vec4">;
}

/**
 * `deffer_detail_w_flat.vs` and `deffer_detail_s_flat.vs` with `deffer_base_aref_flat.ps`: a tuft stood at its place,
 * turned and scaled, a waving one leant by its wave, lit by its slot's sky and sun, and cut out at `def_aref`.
 *
 * @param source - The model's grass.
 * @param samplers - Where its base texture is bound.
 * @param uniforms - What the frame's shaders read.
 * @returns Its shader.
 */
export function toGrassSurfaceShader(
  source: IGrassSurfaceSource,
  samplers: MaterialSamplers,
  uniforms: RendererUniforms
): ISurfaceShader {
  const { items, height, surface } = source;
  const place: Node<"vec4"> = items.element(instanceIndex.mul(2)) as unknown as Node<"vec4">;
  const look: Node<"vec4"> = items.element(instanceIndex.mul(2).add(1)) as unknown as Node<"vec4">;
  const turned: Node<"vec3"> = toTurned(positionLocal.mul(look.x), place.w);
  const standing: Node<"vec3"> = place.xyz.add(turned);
  const wind: GrassWindUniforms = uniforms.grassWind;
  const current: Node<"vec3"> = toSwayed(
    standing,
    place,
    look.w,
    height,
    wind.wind1,
    wind.wave1,
    wind.wind2,
    wind.wave2
  );
  const previous: Node<"vec3"> = toSwayed(
    standing,
    place,
    look.w,
    height,
    wind.previousWind1,
    wind.previousWave1,
    wind.previousWind2,
    wind.previousWave2
  );
  // From a point below the foot to the vertex: a tuft reads lit from above and a little outward, as the engine lights it.
  const normal: Node<"vec3"> = normalize(
    varying(cameraViewMatrix.mul(vec4(normalize(current.sub(place.xyz.sub(vec3(0, NORMAL_DROP, 0)))), 0)).xyz)
  );
  const base: Node<"vec4"> = samplers.bind(surface.textures.base, getWhiteTexture(), uv());
  const albedo: Node<"vec4"> = Fn(() => {
    If(base.w.lessThanEqual(uniform(surface.alphaReference ?? DEFAULT_ALPHA_REFERENCE)), () => {
      Discard();
    });

    return vec4(base.xyz, DEFAULT_GLOSS);
  })();

  return {
    fragmentNode: toGBufferOutput(
      albedo,
      normal,
      varying(look.y),
      varying(look.z),
      uniform((DEFAULT_MATERIAL + 0.5) / MATERIAL_SLICES),
      toPointMotion(uniforms.motion, current, previous)
    ),
    positionViewNode: cameraViewMatrix.mul(vec4(current, 1)).xyz,
  };
}

/** A vertex turned about the tuft's up by its yaw, `Fmatrix::rotateY` carried into renderer space. */
function toTurned(position: Node<"vec3">, yaw: Node<"float">): Node<"vec3"> {
  const c: Node<"float"> = cos(yaw);
  const s: Node<"float"> = sin(yaw);

  return vec3(c.mul(position.x).sub(s.mul(position.z)), position.y, s.mul(position.x).add(c.mul(position.z)));
}

/**
 * `deffer_detail_w_flat.vs`: a waving tuft's vertex leant across the ground by its wave's wind, as far as its height
 * over the foot times the wave at its place, and as much of that as its own height in the model lets it. The wave runs
 * through the engine's space, so the place is read with `z` negated, and the lean carried back the same way.
 */
function toSwayed(
  standing: Node<"vec3">,
  place: Node<"vec4">,
  wave: Node<"float">,
  height: number,
  wind1: Node<"vec3">,
  wave1: Node<"vec4">,
  wind2: Node<"vec3">,
  wave2: Node<"vec4">
): Node<"vec3"> {
  const isSecond: Node<"bool"> = wave.greaterThan(1.5);
  const wind: Node<"vec3"> = select(isSecond, wind2, wind1);
  const phase: Node<"vec4"> = select(isSecond, wave2, wave1);
  const engine: Node<"vec3"> = vec3(standing.x, standing.y, standing.z.negate());
  const cyclic: Node<"float"> = toCyclic(engine.dot(phase.xyz).add(phase.w));
  const share: Node<"float"> = positionLocal.y.div(Math.max(height, 0.0001));
  const lean: Node<"float"> = standing.y.sub(place.y).mul(cyclic).mul(share);
  const swayed: Node<"vec3"> = standing.add(vec3(wind.x.mul(lean), 0, wind.z.mul(lean).negate()));

  return select(wave.greaterThan(0.5), swayed, standing);
}

/** `calc_cyclic`: a wave from minus one to one over each whole turn, a parabola rather than a sine. */
function toCyclic(phase: Node<"float">): Node<"float"> {
  const f: Node<"float"> = fract(phase).mul(2.8284271).sub(1.4142136);

  return f.mul(f).sub(1);
}
