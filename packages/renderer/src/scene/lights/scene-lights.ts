import { Nullable } from "@xrf/types";
import {
  Frustum,
  Matrix4,
  PerspectiveCamera,
  Sphere,
  StorageBufferAttribute,
  TextureNode,
  Vector3,
} from "three/webgpu";

import { IRendererLightsSettings } from "#/contract/renderer-features";
import { TRendererVector } from "#/contract/renderer-lighting";
import { ERendererLightKind, IRendererLight, IRendererLights } from "#/contract/scene/renderer-lights";
import { toSunSpecular } from "#/lighting/base-lighting";
import { toAnimatedColor } from "#/lighting/light-animator";
import { toProjectorAnchor } from "#/scene/lights/light-projectors.tsl";
import { LIGHT_RECORD, LIGHT_VECTORS } from "#/scene/lights/light-record";
import { toLightShadowScale } from "#/scene/lights/light-shadow-faces";
import {
  ILightShadowEntry,
  ILightShadowFace,
  LIGHT_SHADOW_ATLAS_SIZE,
  LightShadows,
} from "#/scene/lights/light-shadows";
import { getWhiteTexture } from "#/texture/placeholder-textures";
import { RendererTextures } from "#/texture/renderer-textures";
import { LIGHT_CLUSTER_CAPACITY, LIGHT_CLUSTERS, LightsUniforms } from "#/uniforms/lights-uniforms";
import { LodUniforms } from "#/uniforms/lod-uniforms";

/** Lights standing in view at most in one frame. */
export const MAX_LIGHTS: number = 1024;

/** Distinct projectors the spots of one scene sample; a spot naming another lights white. */
export const MAX_PROJECTORS: number = 8;

/** Shadow faces drawn at most in one frame, so a level opening fills the atlas over a few frames rather than in one. */
export const LIGHT_SHADOW_FACE_BUDGET: number = 8;

/** What a light's falloff reaches zero at, a share of its range: `L_R` (`r3_rendertarget_accum_point.cpp`). */
const FALLOFF_RANGE: number = 0.95;

/** `ps_r2_slight_fade`: what a shadowed light's screen area is scaled by before it fades (`xrRender_console.cpp`). */
const SHADOWED_FADE: number = 0.5;

/** `EPS_L`: the level of detail a light must pass to be drawn at all. */
const LIGHT_EPSILON: number = 0.001;

/** A cone the record says is none: every point passes its test. */
const NO_CONE: number = -2;

/** A shadowed light's record waiting for the planner to say whether its faces are drawn. */
interface IPendingShadow {
  offset: number;
  entry: ILightShadowEntry;
}

/**
 * A scene's local lights: kept as the consumer put them, and each frame the ones in view written out in view space,
 * animated and faded as the engine would, for the lights pass to bin and accumulate, a shadowed one with the squares
 * of the atlas its faces are drawn in.
 */
export class SceneLights {
  /** `LIGHT_VECTORS` a light standing in view, this frame's. */
  public readonly records: StorageBufferAttribute = new StorageBufferAttribute(
    new Float32Array(MAX_LIGHTS * LIGHT_VECTORS * 4),
    4
  );
  /** Lights reaching each cluster, and which they are. */
  public readonly counts: StorageBufferAttribute = new StorageBufferAttribute(new Uint32Array(LIGHT_CLUSTERS), 1);
  public readonly items: StorageBufferAttribute = new StorageBufferAttribute(
    new Uint32Array(LIGHT_CLUSTERS * LIGHT_CLUSTER_CAPACITY),
    1
  );
  public readonly uniforms: LightsUniforms = new LightsUniforms();
  /** What plans the shadowed lights' faces. */
  public readonly shadows: LightShadows = new LightShadows();
  /** A sampler a projector slot, white where no spot names one. */
  public projectors: ReadonlyArray<TextureNode> = [];
  /** Bumped whenever the projectors are bound again, which the pass sampling them rebuilds on. */
  public version: number = 0;
  /** Lights standing in view this frame. */
  public count: number = 0;

  private readonly textures: RendererTextures;
  private lights: Nullable<IRendererLights> = null;
  /** The key each slot is bound to. */
  private keys: Array<string> = [];
  private readonly frustum: Frustum = new Frustum();
  private readonly viewProjection: Matrix4 = new Matrix4();
  private readonly eye: Vector3 = new Vector3();
  private readonly forward: Vector3 = new Vector3();
  private readonly bound: Sphere = new Sphere();
  private readonly spatial: Sphere = new Sphere();
  private readonly position: Vector3 = new Vector3();
  private readonly vector: Vector3 = new Vector3();
  private readonly direction: Vector3 = new Vector3();
  private readonly right: Vector3 = new Vector3();
  private readonly up: Vector3 = new Vector3();
  private readonly color: Array<number> = [0, 0, 0];
  private pending: Array<IPendingShadow> = [];

  public constructor(textures: RendererTextures) {
    this.textures = textures;
    this.bind([]);
  }

  /** Whether there are lights at all, so a pass lighting them has anything to do. */
  public get isEmpty(): boolean {
    return !this.lights?.lights.length;
  }

  /**
   * @param lights - The scene's lights, replacing any put before.
   */
  public put(lights: IRendererLights): void {
    const keys: Array<string> = [];

    for (const light of lights.lights) {
      if (light.projector && !keys.includes(light.projector) && keys.length < MAX_PROJECTORS) {
        keys.push(light.projector);
      }
    }

    this.lights = lights;
    this.shadows.reset();
    this.bind(keys);
  }

  public release(): void {
    this.lights = null;
    this.count = 0;
    this.shadows.reset();
    this.bind([]);
  }

  /**
   * Writes out the lights standing in view this frame, and plans the shadow faces drawn in it.
   *
   * @param camera - The camera drawing the frame, its matrices current and its projection jittered as it draws.
   * @param time - Seconds, which the animations run by.
   * @param settings - What the lights are set to.
   * @param lod - The level of detail thresholds, which shadowed lights fade by.
   * @param casterVersion - What the shadow casters are at: a face drawn at another is drawn again.
   */
  public update(
    camera: PerspectiveCamera,
    time: number,
    settings: IRendererLightsSettings,
    lod: LodUniforms,
    casterVersion: number
  ): void {
    const data: Float32Array = this.records.array as Float32Array;
    const view: Matrix4 = camera.matrixWorldInverse;
    let count: number = 0;

    this.viewProjection.multiplyMatrices(camera.projectionMatrix, view);
    this.frustum.setFromProjectionMatrix(this.viewProjection, camera.coordinateSystem, camera.reversedDepth);
    camera.getWorldPosition(this.eye);
    camera.getWorldDirection(this.forward);
    this.shadows.begin(casterVersion);
    this.pending = [];

    (this.lights?.lights ?? []).forEach((light: IRendererLight, index: number) => {
      if (count >= MAX_LIGHTS || (light.isLevel && !settings.isLevelLights)) {
        return;
      }

      const fade: number = light.isShadowed ? this.toShadowedFade(light, lod) : 1;

      if (fade <= LIGHT_EPSILON) {
        return;
      }

      this.toBound(light);

      if (!this.frustum.intersectsSphere(this.bound)) {
        return;
      }

      const offset: number = count * LIGHT_VECTORS * 4;

      this.toColor(light, time, light.kind === ERendererLightKind.SPOT ? fade : 1);
      this.toBasis(light);

      if (light.isShadowed && settings.isShadowed) {
        this.requestShadow(index, light, offset);
      }

      this.writeRecord(data, offset, light, view);
      count += 1;
    });

    this.shadows.finish(LIGHT_SHADOW_FACE_BUDGET);
    this.pending.forEach(({ offset, entry }) => this.writeShadow(data, offset, entry));
    this.count = count;
    this.uniforms.follow(camera, count);

    if (count > 0) {
      this.records.clearUpdateRanges();
      this.records.addUpdateRange(0, count * LIGHT_VECTORS * 4);
      this.records.needsUpdate = true;
    }
  }

  public dispose(): void {
    this.release();
  }

  /** Points each projector slot at the key a spot names, and the rest at white. */
  private bind(keys: Array<string>): void {
    if (this.projectors.length && keys.join() === this.keys.join()) {
      return;
    }

    this.projectors.forEach((sampler: TextureNode, slot: number) => this.textures.unbind(this.keys[slot], sampler));
    this.keys = keys;
    this.projectors = Array.from({ length: MAX_PROJECTORS }, (_, slot: number) =>
      this.textures.bind(keys[slot], getWhiteTexture(), toProjectorAnchor())
    );
    this.version += 1;
  }

  /** `light::get_LOD`: a shadowed light fades by its sphere's share of the screen, as the engine's does. */
  private toShadowedFade(light: IRendererLight, lod: LodUniforms): number {
    this.toSpatialSphere(light);

    const distance: number = this.eye.distanceToSquared(this.spatial.center) + LIGHT_EPSILON;
    const area: number = (SHADOWED_FADE * this.spatial.radius) / distance;
    const start: number = lod.glodStart.value;
    const end: number = lod.glodEnd.value;

    return start > end ? Math.sqrt(Math.min(Math.max((area - end) / (start - end), 0), 1)) : 1;
  }

  /** `light::spatial_move`'s sphere, which the engine fades a light and sizes its shadow by. */
  private toSpatialSphere(light: IRendererLight): void {
    const [x, y, z] = light.position;

    this.spatial.center.set(x, y, z);
    this.spatial.radius = light.range;

    if (light.kind !== ERendererLightKind.SPOT) {
      return;
    }

    const half: number = light.cone / 2;
    const isWide: boolean = light.cone >= Math.PI / 2;

    this.spatial.radius = isWide ? light.range * Math.tan(half) : light.range / (2 * Math.cos(half) ** 2);
    this.spatial.center.add(
      toVector(this.vector, light.direction).multiplyScalar(isWide ? light.range : this.spatial.radius)
    );
  }

  /** The least sphere around what a light reaches, which it is culled and binned by. */
  private toBound(light: IRendererLight): void {
    const [x, y, z] = light.position;

    this.bound.center.set(x, y, z);
    this.bound.radius = light.range;

    if (light.kind !== ERendererLightKind.SPOT) {
      return;
    }

    const half: number = Math.min(light.cone / 2, Math.PI / 2);
    // A narrow cone's sphere passes through its apex and its rim; a wide one's is its rim's.
    const offset: number = half > Math.PI / 4 ? light.range * Math.cos(half) : light.range / (2 * Math.cos(half) ** 2);

    this.bound.radius = half > Math.PI / 4 ? light.range * Math.sin(half) : offset;
    this.bound.center.add(toVector(this.vector, light.direction).multiplyScalar(offset));
  }

  private toColor(light: IRendererLight, time: number, fade: number): void {
    const animator = light.animator === undefined ? undefined : this.lights?.animators[light.animator];

    if (animator) {
      toAnimatedColor(animator, time, this.color);

      for (let channel: number = 0; channel < 3; channel += 1) {
        this.color[channel] *= light.animatorScale * fade;
      }
    } else {
      for (let channel: number = 0; channel < 3; channel += 1) {
        this.color[channel] = light.color[channel] * fade;
      }
    }
  }

  /**
   * `compute_xf_spot`: the direction, and the right the lamp gives made square to it through the up they make, in
   * world space. Crossed in the renderer's mirrored space, the engine's `up = dir x right` turns its sign.
   */
  private toBasis(light: IRendererLight): void {
    this.position.set(light.position[0], light.position[1], light.position[2]);
    toVector(this.direction, light.direction).normalize();
    toVector(this.right, light.right);
    this.up.crossVectors(this.direction, this.right).negate().normalize();
    this.right.crossVectors(this.up, this.direction).negate().normalize();
  }

  /** Asks the planner for a shadowed light's faces, sized as the engine sizes its maps, in world space. */
  private requestShadow(index: number, light: IRendererLight, offset: number): void {
    const [red, green, blue] = this.color;
    const isSpot: boolean = light.kind === ERendererLightKind.SPOT;

    this.toSpatialSphere(light);

    const entry: Nullable<ILightShadowEntry> = this.shadows.request(index, {
      cone: light.cone,
      direction: this.direction,
      distance: Math.max(this.eye.distanceTo(this.spatial.center) - this.spatial.radius, 0),
      duel: isSpot ? 1 - 0.5 * this.forward.dot(this.direction) : 1,
      intensity: ((red + green + blue) / 3 + (red * 0.2125 + green * 0.7154 + blue * 0.0721)) / 2,
      isSpot,
      near: light.near,
      position: this.position,
      range: light.range,
      up: this.up,
    });

    if (entry) {
      this.pending.push({ entry, offset });
    }
  }

  private writeRecord(data: Float32Array, offset: number, light: IRendererLight, view: Matrix4): void {
    const isSpot: boolean = light.kind === ERendererLightKind.SPOT;
    const range: number = light.range * FALLOFF_RANGE;
    const slot: number = isSpot && light.projector ? this.keys.indexOf(light.projector) : -1;

    function at(vector: number): number {
      return offset + vector * 4;
    }

    this.vector.copy(this.position).applyMatrix4(view);
    data.set(
      [this.vector.x, this.vector.y, this.vector.z, range > 0 ? 1 / (range * range) : 0],
      at(LIGHT_RECORD.position)
    );
    data.set([this.color[0], this.color[1], this.color[2], toSunSpecular(this.color as never)], at(LIGHT_RECORD.color));
    this.vector.copy(this.direction).transformDirection(view);
    data.set(
      [this.vector.x, this.vector.y, this.vector.z, isSpot ? Math.cos(light.cone / 2) : NO_CONE],
      at(LIGHT_RECORD.axis)
    );
    this.vector.copy(this.right).transformDirection(view);
    data.set(
      [this.vector.x, this.vector.y, this.vector.z, isSpot ? toLightShadowScale(light.cone) : 0],
      at(LIGHT_RECORD.right)
    );
    this.vector.copy(this.up).transformDirection(view);
    data.set([this.vector.x, this.vector.y, this.vector.z, slot], at(LIGHT_RECORD.up));
    this.bound.center.applyMatrix4(view);
    data.set(
      [this.bound.center.x, this.bound.center.y, this.bound.center.z, this.bound.radius],
      at(LIGHT_RECORD.sphere)
    );
    // Unshadowed until the planner says its faces are drawn.
    data.set([0, 0, 0, 0], at(LIGHT_RECORD.shadow));
  }

  /** A shadowed light's near and far planes, and each face's square of the atlas in texture coordinates. */
  private writeShadow(data: Float32Array, offset: number, entry: ILightShadowEntry): void {
    if (!this.shadows.isReady(entry)) {
      return;
    }

    data.set(
      [entry.near, entry.far, entry.faces.length, 1 / LIGHT_SHADOW_ATLAS_SIZE],
      offset + LIGHT_RECORD.shadow * 4
    );
    entry.faces.forEach(({ tile }: ILightShadowFace, face: number) => {
      data.set(
        [tile.x / LIGHT_SHADOW_ATLAS_SIZE, tile.y / LIGHT_SHADOW_ATLAS_SIZE, tile.size / LIGHT_SHADOW_ATLAS_SIZE, 0],
        offset + (LIGHT_RECORD.faces + face) * 4
      );
    });
  }
}

function toVector(out: Vector3, vector: TRendererVector): Vector3 {
  return out.set(vector[0], vector[1], vector[2]);
}
