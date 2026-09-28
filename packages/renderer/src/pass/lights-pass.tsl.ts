import {
  abs,
  clamp,
  dot,
  float,
  floor,
  Fn,
  If,
  int,
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
import { Node, StorageBufferNode, Texture, TextureNode } from "three/webgpu";

import { ERendererLightShadowFilter } from "#/contract/renderer-light-shadow-filter";
import { TRendererVector } from "#/contract/renderer-vector";
import { ILightsPassInputs } from "#/pass/lights-pass-inputs";
import { toLightCluster } from "#/scene/lights/light-clusters.tsl";
import { LIGHT_RECORD, LIGHT_VECTORS, MAX_LIGHTS } from "#/scene/lights/light-record";
import { LIGHT_SHADOW_ATLAS_SIZE } from "#/scene/lights/light-shadow-atlas";
import {
  ILightShadowFaceBasis,
  LIGHT_SHADOW_POINT_CONE,
  LIGHT_SHADOW_POINT_FACES,
  toLightShadowScale,
} from "#/scene/lights/light-shadow-faces";
import { IGBufferSample } from "#/shader/gbuffer-sample";
import { readGBuffer } from "#/shader/gbuffer.tsl";
import { loopNamed } from "#/shader/named-loop.tsl";
import { CameraUniforms } from "#/uniforms/camera-uniforms";
import { LIGHT_CLUSTER_CAPACITY, LIGHT_CLUSTERS, LightsUniforms } from "#/uniforms/lights-uniforms";

/** `gbd.P += gbd.N * 0.015`: the virtual offset `accum_base` moves a point by with the optimised G-buffer. */
const VIRTUAL_OFFSET: number = 0.015;

/** A texel of the atlas, in texture coordinates. */
const ATLAS_TEXEL: number = 1 / LIGHT_SHADOW_ATLAS_SIZE;

/**
 * Every local light reaching a pixel, as the engine's `accum_omni` and `accum_spot` accumulate each:
 * `Ldynamic_color * plight_local(m, P, N) * lightmap * shadow`, a spot's `lightmap` its projector where the pixel stands
 * in its cone, and the shadow of a light that casts one.
 *
 * @param inputs - The G-buffer, the binned lights and what they are shaded with.
 * @param camera - The drawing camera's uniforms.
 * @param uniforms - What the lights were binned by.
 * @param filter - How the shadows are filtered.
 * @returns Diffuse in colour, specular in alpha, to add to what the sun accumulated.
 */
export function toLightsPassFragment(
  inputs: ILightsPassInputs,
  camera: CameraUniforms,
  uniforms: LightsUniforms,
  filter: ERendererLightShadowFilter
): Node<"vec4"> {
  const records: StorageBufferNode<"vec4"> = storage(inputs.records, "vec4", MAX_LIGHTS * LIGHT_VECTORS).toReadOnly();
  const counts: StorageBufferNode<"uint"> = storage(inputs.counts, "uint", LIGHT_CLUSTERS).toReadOnly();
  const items: StorageBufferNode<"uint"> = storage(
    inputs.items,
    "uint",
    LIGHT_CLUSTERS * LIGHT_CLUSTER_CAPACITY
  ).toReadOnly();

  return Fn(() => {
    const sample: IGBufferSample = readGBuffer(inputs.gbuffer, camera);
    const { position, normal, slice } = sample.point;
    const depth: Node<"float"> = position.z.negate();

    const cluster: Node<"uint"> = toLightCluster(uniforms, screenUV, depth);
    const toEye: Node<"vec3"> = normalize(position.negate());
    const offset: Node<"vec3"> = position.add(normal.mul(VIRTUAL_OFFSET));
    const total: Node<"vec4"> = vec4(0).toVar();

    // Nothing drawn there, nothing to light: its depth rebuilds no point.
    const reaching: Node<"uint"> = sample.depth.greaterThan(0).select(counts.element(cluster), uint(0));

    loopNamed({ end: reaching, name: "reaching", start: uint(0), type: "uint" }, (index: Node<"uint">) => {
      const base: Node<"uint"> = items.element(cluster.mul(LIGHT_CLUSTER_CAPACITY).add(index)).mul(LIGHT_VECTORS);
      const place: Node<"vec4"> = records.element(base.add(LIGHT_RECORD.position));
      const color: Node<"vec4"> = records.element(base.add(LIGHT_RECORD.color));
      const axis: Node<"vec4"> = records.element(base.add(LIGHT_RECORD.axis));
      const right: Node<"vec4"> = records.element(base.add(LIGHT_RECORD.right));
      const up: Node<"vec4"> = records.element(base.add(LIGHT_RECORD.up));
      const shadow: Node<"vec4"> = records.element(base.add(LIGHT_RECORD.shadow));
      const isSpot: Node<"bool"> = axis.w.greaterThan(-1);
      const isShadowed: Node<"bool"> = shadow.z.greaterThan(0);
      // `accum_base`, a spot's and a shadowed omni part's, offsets the point; the unshadowed omni shader does not.
      const point: Node<"vec3"> = isSpot.or(isShadowed).select(offset, position);
      const toPoint: Node<"vec3"> = point.sub(place.xyz);
      // `plight_local`: falloff by the squared distance, to zero at 95% of the range.
      const falloff: Node<"float"> = saturate(float(1).sub(dot(toPoint, toPoint).mul(place.w)));
      const lookup: ILightShadowLookup = {
        atlas: inputs.atlas,
        axis,
        base,
        isSpot,
        records,
        right,
        shadow,
        normal,
        toPoint,
        up,
      };

      // Past the light's reach nothing below adds anything: no material, projector or shadow is read there.
      If(falloff.greaterThan(0), () => {
        const toLight: Node<"vec3"> = normalize(toPoint.negate());
        const half: Node<"vec3"> = normalize(toLight.add(toEye));
        const material: Node<"vec4"> = texture3D(
          inputs.lut,
          vec3(dot(toLight, normal), dot(half, normal), slice)
        ).level(float(0));
        const light: Node<"vec4"> = vec4(material.x, material.x, material.x, material.y).mul(falloff).toVar();

        If(isSpot, () => {
          const along: Node<"float"> = dot(toPoint, axis.xyz);
          // In front of the apex and within the cone: the rest of a spot's clusters stays dark, its apex undivided.
          const isInCone: Node<"bool"> = along.greaterThan(0).and(along.greaterThanEqual(axis.w.mul(toPoint.length())));

          If(isInCone, () => {
            const across: Node<"vec2"> = vec2(dot(toPoint, right.xyz), dot(toPoint, up.xyz));

            light.mulAssign(toProjected(inputs.projectors, int(up.w), toFaceUv(across, along, right.w)));

            If(isShadowed, () => {
              light.mulAssign(toLightShadow(lookup, camera, filter));
            });
          }).Else(() => {
            light.assign(vec4(0));
          });
        }).ElseIf(isShadowed, () => {
          light.mulAssign(toLightShadow(lookup, camera, filter));
        });

        total.addAssign(color.mul(light));
      });
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
  /** The point's normal, in view space. */
  normal: Node<"vec3">;
  isSpot: Node<"bool">;
}

/** A point light's face, with the right its camera takes. */
interface IPointFace {
  readonly direction: TRendererVector;
  readonly right: TRendererVector;
  readonly up: TRendererVector;
}

/** A point light's faces, each camera's right `direction x up`. */
const POINT_FACES: ReadonlyArray<IPointFace> = LIGHT_SHADOW_POINT_FACES.map(
  ({ direction, up }: ILightShadowFaceBasis): IPointFace => {
    const [dx, dy, dz] = direction;
    const [ux, uy, uz] = up;

    return { direction, right: [dy * uz - dz * uy, dz * ux - dx * uz, dx * uy - dy * ux], up };
  }
);

/**
 * Texels of its face a point is moved along its normal before it is compared: a departure from the engine, whose maps
 * are as coarse but whose floors around a low light stripe the same way.
 */
const NORMAL_OFFSET: number = 1;

/** `KERNEL`: how far `shadow_hw`'s four taps stand from the point, in texels of the atlas. */
const SHADOW_KERNEL: number = 0.6;

/** `r2_ls_depth_scale`, and each filter's `r2_ls_depth_bias`: what the point's depth is moved by before it is compared. */
const DEPTH_SCALE: number = 1.00001;
const DEPTH_BIAS: Record<ERendererLightShadowFilter, number> = {
  [ERendererLightShadowFilter.ENGINE]: -0.0003,
  [ERendererLightShadowFilter.SOFT]: -0.001,
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
 * in the engine's own depth moved by its scale and bias, and four taps of a bilinear comparison around it. A point's
 * face is its direction's longest axis; a spot has one.
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
  const { atlas, records, base, axis, right, up, shadow, toPoint, normal, isSpot } = lookup;
  // The face's basis and the point, where the face stands: a spot's in view space, a point light's faces in the world.
  const faceRight: Node<"vec3"> = right.xyz.toVar();
  const faceUp: Node<"vec3"> = up.xyz.toVar();
  const faceAxis: Node<"vec3"> = axis.xyz.toVar();
  const point: Node<"vec3"> = toPoint.toVar();
  const bent: Node<"vec3"> = normal.toVar();
  const scale: Node<"float"> = right.w.toVar();
  const face: Node<"uint"> = uint(0).toVar();

  If(isSpot.not(), () => {
    const world: Node<"vec3"> = camera.viewToWorld.mul(vec4(toPoint, 0)).xyz.toVar();
    const size: Node<"vec3"> = abs(world);
    const onX: Node<"bool"> = size.x.greaterThanEqual(size.y).and(size.x.greaterThanEqual(size.z));
    const onY: Node<"bool"> = size.y.greaterThanEqual(size.z);

    face.assign(
      select(
        onX,
        select(world.x.greaterThan(0), uint(0), uint(1)),
        select(onY, select(world.y.greaterThan(0), uint(2), uint(3)), select(world.z.greaterThan(0), uint(4), uint(5)))
      )
    );
    scale.assign(toLightShadowScale(LIGHT_SHADOW_POINT_CONE));
    point.assign(world);
    bent.assign(camera.viewToWorld.mul(vec4(normal, 0)).xyz);
    POINT_FACES.forEach((basis: IPointFace, index: number) => {
      If(face.equal(index), () => {
        faceRight.assign(vec3(...basis.right));
        faceUp.assign(vec3(...basis.up));
        faceAxis.assign(vec3(...basis.direction));
      });
    });
  });

  const rect: Node<"vec4"> = records.element(base.add(LIGHT_RECORD.faces).add(face));
  const texel: Node<"float"> = float(ATLAS_TEXEL);
  const side: Node<"float"> = rect.z.div(texel);
  // A texel of the face across, in metres where the point stands: the face's `2 / scale` of its depth over its texels.
  const reach: Node<"float"> = dot(point, faceAxis)
    .mul(2)
    .div(scale.mul(side.sub(2)));
  const shifted: Node<"vec3"> = point.add(bent.mul(reach.mul(NORMAL_OFFSET))).toVar();
  const across: Node<"vec2"> = vec2(dot(shifted, faceRight), dot(shifted, faceUp));
  const along: Node<"float"> = dot(shifted, faceAxis);
  const near: Node<"float"> = shadow.x;
  const far: Node<"float"> = shadow.y;
  const depth: Node<"float"> = max(along, near);
  // The face's own depth as the engine stores it, `0` near and `1` far, moved as `m_TexelAdjust` moves it; the atlas
  // holds depth reversed, `1` near.
  const engineDepth: Node<"float"> = far.mul(depth.sub(near)).div(depth.mul(far.sub(near)));
  const moved: Node<"float"> = engineDepth.mul(DEPTH_SCALE).add(DEPTH_BIAS[filter]);
  const reference: Node<"float"> = float(1).sub(moved);
  const uv: Node<"vec2"> = toFaceUv(across, along, scale);
  // In texels of the atlas: the face maps into its square a texel in, as the engine maps a face into its sub-rect;
  // each tap is kept inside the square, where the engine's kernel may read a neighbour's texel.
  const corner: Node<"vec2"> = rect.xy.div(texel);
  const least: Node<"vec2"> = corner.add(0.5);
  const most: Node<"vec2"> = corner.add(side).sub(0.5);
  const centre: Node<"vec2"> = corner.add(1).add(uv.mul(side.sub(2)));

  // A point's faces fade each on its own, as the engine's omni parts do.
  const faded: Node<"float"> = rect.w;

  if (filter === ERendererLightShadowFilter.SOFT) {
    return toPenumbraLit(atlas, centre, least, most, texel, moved).mul(faded);
  }

  let lit: Node<"float"> = float(0);

  for (const [x, y] of [
    [-1, -1],
    [1, -1],
    [-1, 1],
    [1, 1],
  ]) {
    const tap: Node<"vec2"> = clamp(centre.add(vec2(x * SHADOW_KERNEL, y * SHADOW_KERNEL)), least, most);

    lit = lit.add(toComparedTexels(atlas, tap, texel, reference));
  }

  return lit.div(4).mul(faded);
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
  const found: Node<"float"> = float(0).toVar();
  const blockers: Node<"float"> = float(0).toVar();
  const texelCentre: Node<"vec2"> = floor(centre).add(0.5);
  const reference: Node<"float"> = float(1).sub(depth);

  for (const row of [-PCSS_PIXEL, 0, PCSS_PIXEL]) {
    for (const column of [-PCSS_PIXEL, 0, PCSS_PIXEL]) {
      const at: Node<"vec2"> = clamp(texelCentre.add(vec2(column, row)), least, most);
      // Held reversed: the engine's own depth is its complement.
      const stored: Node<"float"> = float(1).sub(texture(atlas, at.mul(texel)).level(int(0)).x);
      const isBlocker: Node<"float"> = float(1).sub(step(depth.sub(0.0001), stored));

      blockers.addAssign(isBlocker);
      found.addAssign(stored.mul(isBlocker));
    }
  }

  const lit: Node<"float"> = select(blockers.greaterThanEqual(9), float(0), float(1)).toVar();

  If(blockers.greaterThanEqual(1).and(blockers.lessThan(9)), () => {
    const blocker: Node<"float"> = found.div(blockers);
    const ratio: Node<"float"> = saturate(depth.sub(blocker).mul(PCSS_WIDTH).div(blocker));
    const radius: Node<"float"> = max(float(PCSS_PIXEL_MIN), ratio.mul(ratio).mul(PCSS_PIXEL));
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
  const corner: Node<"vec2"> = at.sub(0.5);
  const first = floor(corner);
  const blend = corner.sub(first);

  function lit(x: number, y: number): Node<"float"> {
    return step(texture(atlas, first.add(vec2(x + 0.5, y + 0.5)).mul(texel)).level(int(0)).x, reference);
  }

  return mix(mix(lit(0, 0), lit(1, 0), blend.x), mix(lit(0, 1), lit(1, 1), blend.x), blend.y);
}

/**
 * `m_Lmap` and a face's projection alike: a point in a light's view, `x` right, `y` up, `along` its direction, over the
 * square the widened cone covers, `v` down.
 *
 * @param across - The point across the light's view.
 * @param along - How far along it.
 * @param scale - `cot` of half the widened cone.
 * @returns Where it stands on the square, from zero to one.
 */
function toFaceUv(across: Node<"vec2">, along: Node<"float">, scale: Node<"float">): Node<"vec2"> {
  return vec2(0.5).add(across.mul(scale).div(along).mul(vec2(0.5, -0.5)));
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
