import { EPS_S } from "@xrf/math";
import { Frustum, PerspectiveCamera, Sphere, Vector3, Vector4 } from "three/webgpu";

import { adoptRendererConventions } from "#/internals/camera-conventions";
import { ILightShadowAsk } from "#/scene/lights/light-shadow-ask";
import { ILightShadowEntry } from "#/scene/lights/light-shadow-entry";
import { ILightShadowFace } from "#/scene/lights/light-shadow-face";
import {
  LIGHT_SHADOW_POINT_CONE,
  LIGHT_SHADOW_POINT_FACES,
  LIGHT_SHADOW_WIDENING,
} from "#/scene/lights/light-shadow-faces";
import { ILightShadowTile } from "#/scene/lights/light-shadow-tile";
import { EShadowCasterMotion } from "#/scene/static/shadow-caster-motion";
import { toCameraFrustum, toPlaneVectors } from "#/visibility/camera-frustum";

/** `EPS_S`: what a face's far plane stands past the light's range. */
const FAR_EPSILON: number = EPS_S;

/** Where a face's projection starts where the light gives none: `light::virtual_size`'s default. */
const DEFAULT_NEAR: number = 0.1;

/**
 * Makes a light's faces over the squares found for them, their cameras built as `compute_xf_spot` builds its own: at
 * the light, down its direction, the cone widened.
 */
export class LightShadowFaceFactory {
  /** What a face's matrices are built with, and its frustum found by. */
  private readonly camera: PerspectiveCamera = new PerspectiveCamera();
  private readonly frustum: Frustum = new Frustum();
  private readonly target: Vector3 = new Vector3();
  /** Bumped by every face made, so a cull run against another face's planes is not taken for its own. */
  private version: number = 0;

  public constructor() {
    adoptRendererConventions(this.camera);
  }

  /**
   * @param ask - The light.
   * @param asked - The square its faces were asked at.
   * @param tiles - A square for each face, all of one size.
   * @param frame - The frame they are made in.
   * @returns Its faces, none drawn yet.
   */
  public create(
    ask: ILightShadowAsk,
    asked: number,
    tiles: ReadonlyArray<ILightShadowTile>,
    frame: number
  ): ILightShadowEntry {
    const near: number = ask.near > 0 ? ask.near : DEFAULT_NEAR;
    const far: number = ask.range + FAR_EPSILON;

    return {
      asked,
      faces: tiles.map((tile: ILightShadowTile, face: number) => this.createFace(ask, face, tile, near, far)),
      far,
      near,
      seen: frame,
      sphere: new Sphere(ask.position.clone(), ask.range),
      size: tiles[0].size,
    };
  }

  private createFace(
    ask: ILightShadowAsk,
    face: number,
    tile: ILightShadowTile,
    near: number,
    far: number
  ): ILightShadowFace {
    const { camera } = this;
    const cone: number = ask.isSpot ? ask.cone : LIGHT_SHADOW_POINT_CONE;
    const planes: Array<Vector4> = Array.from({ length: 6 }, () => new Vector4());

    camera.fov = ((cone + LIGHT_SHADOW_WIDENING) * 180) / Math.PI;
    camera.aspect = 1;
    camera.near = near;
    camera.far = far;
    camera.updateProjectionMatrix();
    camera.position.copy(ask.position);

    if (ask.isSpot) {
      camera.up.copy(ask.up);
      this.target.copy(ask.position).add(ask.direction);
    } else {
      const { direction, up } = LIGHT_SHADOW_POINT_FACES[face];

      camera.up.set(up[0], up[1], up[2]);
      this.target.set(direction[0], direction[1], direction[2]).add(ask.position);
    }

    camera.lookAt(this.target);
    camera.updateMatrixWorld();
    toPlaneVectors(toCameraFrustum(camera, this.frustum).planes, planes);

    return {
      drawnAt: 0,
      far,
      isDrawn: false,
      isStale: false,
      motion: EShadowCasterMotion.STILL,
      near,
      planes,
      projection: camera.projectionMatrix.clone(),
      tile,
      version: ++this.version,
      view: camera.matrixWorldInverse.clone(),
      world: camera.matrixWorld.clone(),
    };
  }
}
