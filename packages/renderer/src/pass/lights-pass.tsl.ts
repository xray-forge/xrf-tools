import {
  abs,
  clamp,
  dot,
  float,
  floor,
  Fn,
  If,
  int,
  log,
  max,
  mix,
  normalize,
  saturate,
  screenUV,
  select,
  step,
  storage,
  texture,
  texture3D,
  uint,
  vec2,
  vec3,
  vec4,
} from "three/tsl";
import { Data3DTexture, Node, StorageBufferAttribute, StorageBufferNode, Texture, TextureNode } from "three/webgpu";

import { ERendererLightShadowFilter } from "#/contract/renderer-features";
import { LIGHT_RECORD } from "#/scene/lights/light-record";
import {
  LIGHT_SHADOW_POINT_CONE,
  LIGHT_SHADOW_POINT_FACES,
  toLightShadowScale,
} from "#/scene/lights/light-shadow-faces";
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
  /** The shadow atlas's depth, reversed. */
  atlas: Texture;
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
  capacity: number,
  filter: ERendererLightShadowFilter
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
      const shadow = records.element(base.add(LIGHT_RECORD.shadow));
      const isSpot = axis.w.greaterThan(-1);
      const isShadowed = shadow.z.greaterThan(0);
      // `accum_base`, a spot's and a shadowed omni part's, offsets the point; the unshadowed omni shader does not.
      const point = isSpot.or(isShadowed).select(offset, position);
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

      If(isShadowed, () => {
        light.mulAssign(
          toLightShadow(
            { atlas: inputs.atlas, axis, base, isSpot, records, right, shadow, toPoint, up },
            camera,
            filter
          )
        );
      });

      total.addAssign(color.mul(light));
    });

    return total;
  })();
}

/** What a shadowed light's lookup reads of its record. */
interface ILightShadowLookup {
  atlas: Texture;
  records: StorageBufferNode<"vec4">;
  /** Where the light's record starts. */
  base: Node<"uint">;
  axis: Node<"vec4">;
  right: Node<"vec4">;
  up: Node<"vec4">;
  shadow: Node<"vec4">;
  /** The point from the light, in view space. */
  toPoint: Node<"vec3">;
  isSpot: Node<"bool">;
}

/** A point light's faces, with the right each camera takes: `direction x up`. */
const POINT_FACES = LIGHT_SHADOW_POINT_FACES.map(({ direction, up }) => {
  const [dx, dy, dz] = direction;
  const [ux, uy, uz] = up;

  return { direction, right: [dy * uz - dz * uy, dz * ux - dx * uz, dx * uy - dy * ux] as const, up };
});

/** `KERNEL`: how far `shadow_hw`'s four taps stand from the point, in texels of the atlas. */
const SHADOW_KERNEL: number = 0.6;

/** `r2_ls_depth_scale`, and each filter's `r2_ls_depth_bias`: what the point's depth is moved by before it is compared. */
const DEPTH_SCALE: number = 1.00001;
const DEPTH_BIAS: Record<ERendererLightShadowFilter, number> = {
  [ERendererLightShadowFilter.ENGINE]: -0.0003,
  [ERendererLightShadowFilter.ANOMALY]: -0.001,
};

/** Anomaly's `poissonDisk`: the first twelve, which its `shadow_pcss` takes at its default quality. */
const POISSON_DISK: ReadonlyArray<readonly [number, number]> = [
  [0.0617981, 0.07294159],
  [0.6470215, 0.7474022],
  [-0.5987766, -0.7512833],
  [-0.693034, 0.6913887],
  [0.6987045, -0.6843052],
  [-0.9402866, 0.04474335],
  [0.8934509, 0.07369385],
  [0.1592735, -0.9686295],
  [-0.05664673, 0.995282],
  [-0.1203411, -0.1301079],
  [0.1741608, -0.1682285],
  [-0.09369049, 0.3196758],
];

/** `PCSS_PIXEL`, `PCSS_PIXEL_MIN` and `PCSS_SUN_WIDTH`: how far the blockers are searched, and the penumbra's scale. */
const PCSS_PIXEL: number = 5;
const PCSS_PIXEL_MIN: number = 1;
const PCSS_WIDTH: number = 150;

/**
 * How much of a shadowed light reaches a point, as `shadow_hw` finds it: the face the point stands in, its depth there
 * in the engine's own depth moved by its scale and bias, and four taps of a bilinear comparison around it, kept a texel
 * and a half inside the face's square. A point's face is its direction's longest axis; a spot has one.
 *
 * @param lookup - The light's record, and the point.
 * @param camera - The drawing camera's uniforms, which turn the point into the world a point light's faces stand in.
 * @param filter - How the comparison is filtered.
 * @returns The light's share, from nothing to one.
 */
function toLightShadow(
  lookup: ILightShadowLookup,
  camera: CameraUniforms,
  filter: ERendererLightShadowFilter
): Node<"float"> {
  const { atlas, records, base, axis, right, up, shadow, toPoint, isSpot } = lookup;
  const across = vec2(dot(toPoint, right.xyz), dot(toPoint, up.xyz)).toVar();
  const along = dot(toPoint, axis.xyz).toVar();
  const scale = right.w.toVar();
  const face = uint(0).toVar();

  If(isSpot.not(), () => {
    const world = camera.viewToWorld.mul(vec4(toPoint, 0)).xyz.toVar();
    const size = abs(world);
    const onX = size.x.greaterThanEqual(size.y).and(size.x.greaterThanEqual(size.z));
    const onY = size.y.greaterThanEqual(size.z);

    face.assign(
      select(
        onX,
        select(world.x.greaterThan(0), uint(0), uint(1)),
        select(onY, select(world.y.greaterThan(0), uint(2), uint(3)), select(world.z.greaterThan(0), uint(4), uint(5)))
      )
    );
    scale.assign(toLightShadowScale(LIGHT_SHADOW_POINT_CONE));
    POINT_FACES.forEach((basis, index: number) => {
      If(face.equal(index), () => {
        across.assign(vec2(dot(world, vec3(...basis.right)), dot(world, vec3(...basis.up))));
        along.assign(dot(world, vec3(...basis.direction)));
      });
    });
  });

  const rect = records.element(base.add(LIGHT_RECORD.faces).add(face));
  const texel = shadow.w;
  const [near, far] = [shadow.x, shadow.y];
  const depth = max(along, near);
  // The face's own depth as the engine stores it, `0` near and `1` far, moved as `m_TexelAdjust` moves it; the atlas
  // holds depth reversed, `1` near.
  const engineDepth = far.mul(depth.sub(near)).div(depth.mul(far.sub(near)));
  const moved = engineDepth.mul(DEPTH_SCALE).add(DEPTH_BIAS[filter]);
  const reference = float(1).sub(moved);
  const uv = vec2(0.5).add(across.mul(scale).div(along).mul(vec2(0.5, -0.5)));
  // In texels of the atlas, the square's own edge a texel and a half in, as the engine insets its sub-rect.
  const least = rect.xy.div(texel).add(1.5);
  const most = rect.xy.add(rect.zz).div(texel).sub(1.5);
  const centre = clamp(rect.xy.add(uv.mul(rect.z)).div(texel), least, most);

  if (filter === ERendererLightShadowFilter.ANOMALY) {
    return toPenumbraLit(atlas, centre, least, most, texel, moved);
  }

  let lit: Node<"float"> = float(0);

  for (const [x, y] of [
    [-1, -1],
    [1, -1],
    [-1, 1],
    [1, 1],
  ]) {
    lit = lit.add(toComparedTexels(atlas, centre.add(vec2(x * SHADOW_KERNEL, y * SHADOW_KERNEL)), texel, reference));
  }

  return lit.div(4);
}

/**
 * Anomaly's `shadow_pcss`: nine texels five apart searched for what stands nearer the light, lit where none does and
 * dark where all do; between, a penumbra of comparisons as wide as the blockers stand from the point, at least a texel.
 *
 * @param atlas - The atlas's depth.
 * @param centre - The point, in texels.
 * @param least - The face's square's least texel a comparison may take.
 * @param most - And its most.
 * @param texel - A texel's width in texture coordinates.
 * @param depth - The point's depth in the engine's own depth, `0` near, moved by its scale and bias.
 * @returns The lit share.
 */
function toPenumbraLit(
  atlas: Texture,
  centre: Node<"vec2">,
  least: Node<"vec2">,
  most: Node<"vec2">,
  texel: Node<"float">,
  depth: Node<"float">
): Node<"float"> {
  const found = float(0).toVar();
  const blockers = float(0).toVar();
  const texelCentre = floor(centre).add(0.5);
  const reference = float(1).sub(depth);

  for (const row of [-PCSS_PIXEL, 0, PCSS_PIXEL]) {
    for (const column of [-PCSS_PIXEL, 0, PCSS_PIXEL]) {
      const at = clamp(texelCentre.add(vec2(column, row)), least, most);
      // Held reversed: the engine's own depth is its complement.
      const stored = float(1).sub(texture(atlas, at.mul(texel)).level(int(0)).x);
      const isBlocker = float(1).sub(step(depth.sub(0.0001), stored));

      blockers.addAssign(isBlocker);
      found.addAssign(stored.mul(isBlocker));
    }
  }

  const lit = select(blockers.greaterThanEqual(9), float(0), float(1)).toVar();

  If(blockers.greaterThanEqual(1).and(blockers.lessThan(9)), () => {
    const blocker = found.div(blockers);
    const ratio = saturate(depth.sub(blocker).mul(PCSS_WIDTH).div(blocker));
    const radius = max(float(PCSS_PIXEL_MIN), ratio.mul(ratio).mul(PCSS_PIXEL));
    let total: Node<"float"> = float(0);

    for (const [x, y] of POISSON_DISK) {
      total = total.add(
        toComparedTexels(atlas, clamp(centre.add(vec2(x, y).mul(radius)), least, most), texel, reference)
      );
    }

    lit.assign(total.div(POISSON_DISK.length));
  });

  return lit;
}

/**
 * A comparison filtered as hardware filters it, between the four texels around a point: lit where the stored depth is
 * no nearer than the reference.
 *
 * @param atlas - The atlas's depth.
 * @param at - The point, in texels.
 * @param texel - A texel's width in texture coordinates.
 * @param reference - The point's depth, reversed.
 * @returns The lit share.
 */
function toComparedTexels(
  atlas: Texture,
  at: Node<"vec2">,
  texel: Node<"float">,
  reference: Node<"float">
): Node<"float"> {
  const corner = at.sub(0.5);
  const first = floor(corner);
  const blend = corner.sub(first);

  function lit(x: number, y: number): Node<"float"> {
    return step(texture(atlas, first.add(vec2(x + 0.5, y + 0.5)).mul(texel)).level(int(0)).x, reference);
  }

  return mix(mix(lit(0, 0), lit(1, 0), blend.x), mix(lit(0, 1), lit(1, 1), blend.x), blend.y);
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
