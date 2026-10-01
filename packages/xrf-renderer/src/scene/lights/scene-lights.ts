import { EPS_L } from "@xrf/math";
import { Maybe, Nullable } from "@xrf/types";
import { Frustum, Matrix4, Sphere, Vector3, WebGPURenderer } from "three/webgpu";

import { IRendererLightsReport } from "#/contract/renderer-lights-report";
import { ERendererLightKind, TRendererLight } from "#/contract/scene/renderer-light";
import { IRendererLightAnimator } from "#/contract/scene/renderer-light-animator";
import { IRendererLights } from "#/contract/scene/renderer-lights";
import { IRendererSpotLight } from "#/contract/scene/renderer-spot-light";
import { toSunSpecular } from "#/lighting/base-lighting";
import { toAnimatedColor } from "#/lighting/light-animator";
import { ILightBasis } from "#/scene/lights/light-basis";
import { LightClusters } from "#/scene/lights/light-clusters";
import {
  createLightBasis,
  toLightBasis,
  toLightBound,
  toLightFaceSphere,
  toLightIntensity,
  toLightLod,
  toLightSpatialSphere,
} from "#/scene/lights/light-geometry";
import { LightProjectors } from "#/scene/lights/light-projectors";
import { LIGHT_NO_CONE, LIGHT_RECORD, MAX_LIGHTS } from "#/scene/lights/light-record";
import { LightRecords } from "#/scene/lights/light-records";
import { LIGHT_SHADOW_ATLAS_SIZE } from "#/scene/lights/light-shadow-atlas";
import { ILightShadowEntry } from "#/scene/lights/light-shadow-entry";
import { ILightShadowFace } from "#/scene/lights/light-shadow-face";
import { ILightShadowFaceBasis } from "#/scene/lights/light-shadow-face-basis";
import { LIGHT_SHADOW_POINT_FACES, toLightShadowScale } from "#/scene/lights/light-shadow-faces";
import { LightShadowPlanner } from "#/scene/lights/light-shadow-planner";
import { ILightShadowRequest } from "#/scene/lights/light-shadow-request";
import { byDistance, takeSorted } from "#/scene/lights/light-sorting";
import { ISceneLightsFrame } from "#/scene/lights/scene-lights-frame";
import { StaticShadowChanges } from "#/scene/static/static-shadow-changes";
import { RendererTextures } from "#/texture/renderer-textures";
import { LIGHT_SHADOW_FACE_BUDGET } from "#/uniforms/lights-uniforms";
import { LodUniforms } from "#/uniforms/lod-uniforms";
import { toCameraFrustum } from "#/visibility/camera-frustum";

/** What a light's falloff reaches zero at, a share of its range: `L_R` (`r3_rendertarget_accum_point.cpp`). */
const FALLOFF_RANGE: number = 0.95;

/** A light standing in view this frame. */
interface IInViewLight {
  light: TRendererLight;
  /** Its place among the scene's lights, which its shadow is planned under. */
  index: number;
  /** Metres from the eye to its bound's edge, which the nearest are kept by. */
  distance: number;
  /** How far its colour has faded, one for none: a shadowed spot's, as the engine fades it whole. */
  fade: number;
  /** How far each face of a shadowed point has faded, as the engine fades each omni part on its own; ones otherwise. */
  faceFades: Array<number>;
}

/**
 * A scene's local lights: kept as the consumer put them, and each frame the nearest in view written out in view space,
 * animated and faded as the engine would, for the lights pass to bin and accumulate; a shadowed one only once its
 * faces are drawn, with the squares of the atlas they are drawn in.
 */
export class SceneLights {
  public readonly records: LightRecords = new LightRecords();
  public readonly clusters: LightClusters = new LightClusters(this.records.buffer);
  public readonly projectors: LightProjectors;
  public readonly shadows: LightShadowPlanner;
  /** Lights standing in view this frame, and of them the ones drawn with their shadows. */
  public count: number = 0;
  public shadowed: number = 0;
  /** Lights in view past the ones the records hold, the farthest. */
  public excess: number = 0;

  private readonly random: () => number;
  private lights: Nullable<IRendererLights> = null;
  /** Whether the last frame drew the lights' shadows: the atlas goes with the pass while they are off. */
  private wasShadowing: boolean = false;
  private readonly inView: Array<IInViewLight> = [];
  /** The first of them this frame, nearest first, sorted in an array kept between frames. */
  private readonly visible: Array<IInViewLight> = [];
  private readonly frustum: Frustum = new Frustum();
  private readonly eye: Vector3 = new Vector3();
  private readonly forward: Vector3 = new Vector3();
  private readonly bound: Sphere = new Sphere();
  private readonly spatial: Sphere = new Sphere();
  private readonly basis: ILightBasis = createLightBasis();
  private readonly vector: Vector3 = new Vector3();
  private readonly color: [number, number, number] = [0, 0, 0];
  private readonly request: ILightShadowRequest = {
    cone: 0,
    direction: this.basis.direction,
    distance: 0,
    duel: 1,
    intensity: 0,
    isSpot: false,
    near: 0,
    position: this.basis.position,
    range: 0,
    up: this.basis.up,
  };

  /**
   * @param textures - What the spots' projectors are bound through.
   * @param changes - Where what the shadow views draw changed, which the shadow faces kept are drawn again by.
   * @param random - What a zone's range strays by each frame, from zero to one.
   */
  public constructor(textures: RendererTextures, changes: StaticShadowChanges, random: () => number = Math.random) {
    this.projectors = new LightProjectors(textures);
    this.shadows = new LightShadowPlanner(changes);
    this.random = random;
  }

  /** What the lights came to in the last frame, the clusters' fill as last read back. */
  public get report(): IRendererLightsReport {
    return {
      atlas: { capacity: LIGHT_SHADOW_ATLAS_SIZE * LIGHT_SHADOW_ATLAS_SIZE, used: this.shadows.atlas.used },
      droppedLights: this.clusters.droppedLights,
      excessLights: this.excess,
      fullClusters: this.clusters.fullClusters,
      inView: this.count,
      shadowScale: this.shadows.sizeScale,
      shadowed: this.shadowed,
    };
  }

  /**
   * @param lights - The scene's lights, replacing any put before.
   */
  public put(lights: IRendererLights): void {
    this.lights = lights;
    this.shadows.reset();
    this.projectors.put(lights.lights);
  }

  public release(): void {
    this.lights = null;
    this.count = 0;
    this.shadowed = 0;
    this.excess = 0;
    this.shadows.reset();
    this.projectors.release();
    this.clusters.forget();
  }

  /**
   * Writes out the lights standing in view this frame, nearest first, and plans the shadow faces drawn in it.
   *
   * @param frame - What the frame's lights are written for.
   */
  public update(frame: ISceneLightsFrame): void {
    const { view, camera, settings } = frame;

    this.count = 0;
    this.shadowed = 0;
    this.excess = 0;

    const isShadowing: boolean = settings.isEnabled && settings.isShadowed && this.lights !== null;

    // Shadows back on draw into an atlas allocated again: every face is drawn again before its light lights.
    if (isShadowing && !this.wasShadowing) {
      this.shadows.forgetDrawn();
    }

    this.wasShadowing = isShadowing;

    if (settings.isEnabled && this.lights) {
      view.getWorldPosition(this.eye);
      view.getWorldDirection(this.forward);
      toCameraFrustum(view, this.frustum);

      const inView: Array<IInViewLight> = this.findInView(this.lights.lights, frame);

      if (isShadowing) {
        this.shadows.begin(frame.isWindy, frame.time);

        // Only the nearest the records could hold ask for faces.
        for (let at: number = 0; at < Math.min(inView.length, MAX_LIGHTS); at += 1) {
          const { light, index }: IInViewLight = inView[at];

          if (light.isShadowed) {
            this.requestShadow(index, light);
          }
        }

        this.shadows.finish(LIGHT_SHADOW_FACE_BUDGET);
      }

      for (const { light, index, fade, faceFades } of inView) {
        if (this.count === MAX_LIGHTS) {
          this.excess += 1;
          continue;
        }

        const entry: Nullable<ILightShadowEntry> =
          isShadowing && light.isShadowed ? this.shadows.getEntry(index) : null;

        // The engine never lights a shadowed light without its map: it waits for its faces, leaving its record to the
        // next light in view.
        if (isShadowing && light.isShadowed && !entry) {
          continue;
        }

        this.writeLight(this.count, light, fade, frame.time, camera.matrixWorldInverse);
        this.writeShadow(this.count, entry, faceFades);
        this.count += 1;
      }
    }

    this.clusters.follow(camera, this.count);
    this.records.upload(this.count);
  }

  /**
   * @param renderer - The renderer the clusters were binned by.
   */
  public readClusterDrops(renderer: WebGPURenderer): void {
    this.clusters.readDrops(renderer);
  }

  public dispose(): void {
    this.release();
    this.projectors.dispose();
    this.clusters.dispose();
  }

  /** The lights the view sees, not faded out, nearest first, in objects kept for the next frame. */
  private findInView(lights: ReadonlyArray<TRendererLight>, frame: ISceneLightsFrame): Array<IInViewLight> {
    const { lod, settings } = frame;
    let count: number = 0;

    lights.forEach((light: TRendererLight, index: number) => {
      if (light.isLevel && !settings.isLevelLights) {
        return;
      }

      const entry: IInViewLight = (this.inView[count] ??= {
        distance: 0,
        faceFades: LIGHT_SHADOW_POINT_FACES.map(() => 1),
        fade: 1,
        index: 0,
        light,
      });

      if (!this.frustum.intersectsSphere(toLightBound(light, this.bound))) {
        return;
      }

      // `light::get_LOD`, a light the engine shadows alone: by its sphere's share of the screen, a point's each face,
      // drawn at all only past `EPS_L`.
      if (this.toFade(light, entry, settings.isShadowed, lod) <= EPS_L) {
        return;
      }

      entry.light = light;
      entry.index = index;
      entry.distance = Math.max(this.eye.distanceTo(this.bound.center) - this.bound.radius, 0);
      count += 1;
    });

    return takeSorted(this.inView, count, this.visible, byDistance);
  }

  /**
   * Writes how far a light has faded into its entry, a shadowed point's face by face.
   *
   * @returns The most any of it shows, which it is culled by.
   */
  private toFade(light: TRendererLight, entry: IInViewLight, isShadowing: boolean, lod: LodUniforms): number {
    const { glodStart, glodEnd } = lod;

    entry.fade = 1;
    entry.faceFades.fill(1);

    if (!isShadowing || !light.isShadowed) {
      return 1;
    }

    if (light.kind === ERendererLightKind.SPOT) {
      entry.fade = toLightLod(toLightSpatialSphere(light, this.spatial), this.eye, glodStart.value, glodEnd.value);

      return entry.fade;
    }

    let shown: number = 0;

    LIGHT_SHADOW_POINT_FACES.forEach(({ direction }: ILightShadowFaceBasis, face: number) => {
      const fade: number = toLightLod(
        toLightFaceSphere(light, direction, this.spatial),
        this.eye,
        glodStart.value,
        glodEnd.value
      );

      entry.faceFades[face] = fade;
      shown = Math.max(shown, fade);
    });

    return shown;
  }

  /** Asks the planner for a shadowed light's faces, sized as the engine sizes its maps, by its colour unfaded. */
  private requestShadow(index: number, light: TRendererLight): void {
    const { request } = this;
    const spot: Nullable<IRendererSpotLight> = light.kind === ERendererLightKind.SPOT ? light : null;

    toLightBasis(light, this.basis);
    toLightSpatialSphere(light, this.spatial);
    request.isSpot = spot !== null;
    request.cone = spot?.cone ?? 0;
    request.near = light.near;
    // As far as its range strays: past a face's far plane every point reads as shadowed.
    request.range = light.range + (light.rangeJitter ?? 0);
    request.intensity = toLightIntensity(light.color);
    request.distance = Math.max(this.eye.distanceTo(this.spatial.center) - this.spatial.radius, 0);
    request.duel = spot ? 1 - 0.5 * this.forward.dot(this.basis.direction) : 1;
    this.shadows.request(index, request);
  }

  /** A light's record, in view space, its colour animated and faded. */
  private writeLight(slot: number, light: TRendererLight, fade: number, time: number, view: Matrix4): void {
    const { records, basis, vector, color } = this;
    const spot: Nullable<IRendererSpotLight> = light.kind === ERendererLightKind.SPOT ? light : null;
    const range: number = this.toFrameRange(light) * FALLOFF_RANGE;

    toLightBasis(light, basis);
    this.toColor(light, time, fade);
    records.setVector(
      slot,
      LIGHT_RECORD.position,
      vector.copy(basis.position).applyMatrix4(view),
      range > 0 ? 1 / (range * range) : 0
    );
    records.set(slot, LIGHT_RECORD.color, color[0], color[1], color[2], toSunSpecular(color));
    records.setVector(
      slot,
      LIGHT_RECORD.axis,
      vector.copy(basis.direction).transformDirection(view),
      spot ? Math.cos(spot.cone / 2) : LIGHT_NO_CONE
    );
    records.setVector(
      slot,
      LIGHT_RECORD.right,
      vector.copy(basis.right).transformDirection(view),
      spot ? toLightShadowScale(spot.cone) : 0
    );
    records.setVector(
      slot,
      LIGHT_RECORD.up,
      vector.copy(basis.up).transformDirection(view),
      this.projectors.getSlot(light)
    );
    toLightBound(light, this.bound).center.applyMatrix4(view);
    records.setVector(slot, LIGHT_RECORD.sphere, this.bound.center, this.bound.radius);
  }

  /**
   * A shadowed light's near and far planes and face count, and each face's square of the atlas in texture coordinates
   * with how far the face has faded.
   */
  private writeShadow(slot: number, entry: Nullable<ILightShadowEntry>, faceFades: ReadonlyArray<number>): void {
    if (!entry) {
      this.records.set(slot, LIGHT_RECORD.shadow, 0, 0, 0, 0);

      return;
    }

    this.shadowed += 1;
    this.records.set(slot, LIGHT_RECORD.shadow, entry.near, entry.far, entry.faces.length, 0);
    entry.faces.forEach(({ tile }: ILightShadowFace, face: number) =>
      this.records.setFace(
        slot,
        face,
        tile.x / LIGHT_SHADOW_ATLAS_SIZE,
        tile.y / LIGHT_SHADOW_ATLAS_SIZE,
        tile.size / LIGHT_SHADOW_ATLAS_SIZE,
        faceFades[face]
      )
    );
  }

  /** Its colour this frame: animated where it names an animation, then faded. */
  private toColor(light: TRendererLight, time: number, fade: number): void {
    const animator: Maybe<IRendererLightAnimator> =
      light.animator === undefined ? undefined : this.lights?.animators[light.animator];
    const { color } = this;

    if (animator) {
      toAnimatedColor(animator, time, color);

      for (let channel: number = 0; channel < 3; channel += 1) {
        color[channel] *= light.animatorScale * fade;
      }
    } else {
      for (let channel: number = 0; channel < 3; channel += 1) {
        color[channel] = light.color[channel] * fade;
      }
    }
  }

  /** `UpdateIdleLight`: a light's range this frame, strayed at random by its jitter. */
  private toFrameRange(light: TRendererLight): number {
    return light.rangeJitter ? light.range + light.rangeJitter * (this.random() * 2 - 1) : light.range;
  }
}
