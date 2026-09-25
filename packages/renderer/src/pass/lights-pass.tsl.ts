import {
  clamp,
  dot,
  float,
  floor,
  Fn,
  If,
  int,
  log,
  normalize,
  saturate,
  screenUV,
  storage,
  texture3D,
  uint,
  vec2,
  vec3,
  vec4,
} from "three/tsl";
import { Data3DTexture, Node, StorageBufferAttribute, TextureNode } from "three/webgpu";

import { IGBufferSample } from "#/shader/gbuffer-sample";
import { IGBufferTextures } from "#/shader/gbuffer-textures";
import { readGBuffer } from "#/shader/gbuffer.tsl";
import { loopNamed } from "#/shader/named-loop.tsl";
import { CameraUniforms } from "#/uniforms/camera-uniforms";
import {
  LIGHT_CLUSTER_CAPACITY,
  LIGHT_CLUSTERS,
  LIGHT_CLUSTERS_X,
  LIGHT_CLUSTERS_Y,
  LIGHT_CLUSTERS_Z,
  LightsUniforms,
} from "#/uniforms/lights-uniforms";

/** What the lights are accumulated from. */
export interface ILightsPassInputs {
  gbuffer: IGBufferTextures;
  records: StorageBufferAttribute;
  counts: StorageBufferAttribute;
  items: StorageBufferAttribute;
  lut: Data3DTexture;
  /** A sampler a projector slot. */
  projectors: ReadonlyArray<TextureNode>;
}

/** `gbd.P += gbd.N * 0.015`: the virtual offset `accum_base` moves a point by with the optimised G-buffer. */
const VIRTUAL_OFFSET: number = 0.015;

/**
 * Every local light reaching a pixel, as `accum_omni_unshadowed` and `accum_spot_unshadowed` accumulate each:
 * `Ldynamic_color * plight_local(m, P, N) * lightmap`, a spot's `lightmap` its projector where the pixel stands in its
 * cone.
 *
 * @param inputs - The G-buffer, the binned lights and what they are shaded with.
 * @param camera - The drawing camera's uniforms.
 * @param uniforms - What the lights were binned by.
 * @param vectors - Vectors of four floats a light record takes.
 * @param capacity - Lights the records hold.
 * @returns Diffuse in colour, specular in alpha, to add to what the sun accumulated.
 */
export function toLightsPassFragment(
  inputs: ILightsPassInputs,
  camera: CameraUniforms,
  uniforms: LightsUniforms,
  vectors: number,
  capacity: number
): Node<"vec4"> {
  const records = storage(inputs.records, "vec4", capacity * vectors).toReadOnly();
  const counts = storage(inputs.counts, "uint", LIGHT_CLUSTERS).toReadOnly();
  const items = storage(inputs.items, "uint", LIGHT_CLUSTERS * LIGHT_CLUSTER_CAPACITY).toReadOnly();

  return Fn(() => {
    const sample: IGBufferSample = readGBuffer(inputs.gbuffer, camera);
    const { position, normal, slice } = sample.point;
    const depth = position.z.negate();

    const tile = clamp(
      floor(screenUV.mul(vec2(LIGHT_CLUSTERS_X, LIGHT_CLUSTERS_Y))),
      vec2(0),
      vec2(LIGHT_CLUSTERS_X - 1, LIGHT_CLUSTERS_Y - 1)
    );
    const depthSlice = clamp(
      floor(
        log(depth.div(uniforms.near))
          .div(log(uniforms.far.div(uniforms.near)))
          .mul(LIGHT_CLUSTERS_Z)
      ),
      0,
      LIGHT_CLUSTERS_Z - 1
    );
    const cluster = uint(tile.x)
      .add(uint(tile.y).mul(LIGHT_CLUSTERS_X))
      .add(uint(depthSlice).mul(LIGHT_CLUSTERS_X * LIGHT_CLUSTERS_Y));
    const toEye = normalize(position.negate());
    const offset = position.add(normal.mul(VIRTUAL_OFFSET));
    const total = vec4(0).toVar();

    // Nothing drawn there, nothing to light: its depth rebuilds no point.
    const reaching = sample.depth.greaterThan(0).select(counts.element(cluster), uint(0));

    loopNamed({ end: reaching, name: "reaching", start: uint(0), type: "uint" }, (index) => {
      const base = items.element(cluster.mul(LIGHT_CLUSTER_CAPACITY).add(index)).mul(vectors);
      const place = records.element(base);
      const color = records.element(base.add(1));
      const axis = records.element(base.add(2));
      const right = records.element(base.add(3));
      const up = records.element(base.add(4));
      const isSpot = axis.w.greaterThan(-1);
      // `accum_base` offsets the point; the unshadowed omni shader does not.
      const point = isSpot.select(offset, position);
      const toPoint = point.sub(place.xyz);
      // `plight_local`: falloff by the squared distance, to zero at 95% of the range.
      const falloff = saturate(float(1).sub(dot(toPoint, toPoint).mul(place.w)));
      const toLight = normalize(toPoint.negate());
      const half = normalize(toLight.add(toEye));
      const material = texture3D(inputs.lut, vec3(dot(toLight, normal), dot(half, normal), slice)).level(float(0));
      const light = vec4(material.x, material.x, material.x, material.y).mul(falloff).toVar();

      If(isSpot, () => {
        // The point in the light's own view: `x` right, `y` up, `z` along its direction.
        const along = dot(toPoint, axis.xyz);
        const across = vec2(dot(toPoint, right.xyz), dot(toPoint, up.xyz));
        const isInCone = along.greaterThan(0).and(along.greaterThanEqual(axis.w.mul(toPoint.length())));
        // `m_Lmap`: the widened projection, `v` down, over the texture's square.
        const uv = vec2(0.5).add(across.mul(right.w).div(along).mul(vec2(0.5, -0.5)));

        light.mulAssign(isInCone.select(toProjected(inputs.projectors, int(up.w), uv), vec4(0)));
      });

      total.addAssign(color.mul(light));
    });

    return total;
  })();
}

/**
 * @param projectors - A sampler a slot.
 * @param slot - The slot a spot samples, or less than zero for none.
 * @param uv - Where on the projector.
 * @returns The projector's texel, white for a spot without one.
 */
function toProjected(projectors: ReadonlyArray<TextureNode>, slot: Node<"int">, uv: Node<"vec2">): Node<"vec4"> {
  const texel = vec4(1).toVar();

  projectors.forEach((projector: TextureNode, index: number) => {
    If(slot.equal(index), () => {
      texel.assign(projector.sample(uv).level(float(0)));
    });
  });

  return texel;
}
