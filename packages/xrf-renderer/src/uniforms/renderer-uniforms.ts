import { Nullable } from "@xrf/types";
import { Data3DTexture, PerspectiveCamera, WebGPURenderer } from "three/webgpu";

import { IRendererLighting } from "#/contract/renderer-lighting";
import { IRendererSettings } from "#/contract/renderer-settings";
import { toBaseLightingConstants } from "#/lighting/base-lighting";
import { BaseLightingUniforms } from "#/uniforms/base-lighting-uniforms";
import { CameraUniforms } from "#/uniforms/camera-uniforms";
import { CloudUniforms } from "#/uniforms/cloud-uniforms";
import { ExposureUniforms } from "#/uniforms/exposure-uniforms";
import { GrassWindUniforms } from "#/uniforms/grass-wind-uniforms";
import { createMaterialLutTexture } from "#/uniforms/material-lut-texture";
import { MotionUniforms } from "#/uniforms/motion-uniforms";
import { SettingsUniforms } from "#/uniforms/settings-uniforms";
import { ShadowUniforms } from "#/uniforms/shadow-uniforms";
import { SkyUniforms } from "#/uniforms/sky-uniforms";
import { StaticDrawBuffers } from "#/uniforms/static-draw-buffers";
import { StorageRetirement } from "#/uniforms/storage-retirement";
import { SurfaceTable } from "#/uniforms/surface-table";
import { TreeWindUniforms } from "#/uniforms/tree-wind-uniforms";
import { WaterUniforms } from "#/uniforms/water-uniforms";

/**
 * Everything the frame's shaders read besides the scene: uniforms updated in place, so no change recompiles a
 * material, the material lookup every lit shader samples, and the buffers static draws are placed by. The uniforms sit
 * in three's render group, refreshed once a render call rather than per object, which a static draw's replay relies on.
 */
export class RendererUniforms {
  public readonly camera: CameraUniforms = new CameraUniforms();
  public readonly lighting: BaseLightingUniforms = new BaseLightingUniforms();
  public readonly settings: SettingsUniforms = new SettingsUniforms();
  /** What every tonemap multiplies by, adapted to the frame. */
  public readonly exposure: ExposureUniforms = new ExposureUniforms(this.settings.tonemapScale);
  public readonly lut: Data3DTexture = createMaterialLutTexture();
  /** Storage let go of by any part, freed a frame later. */
  public readonly retirement: StorageRetirement = new StorageRetirement();
  /** What every static draw is culled and placed by. */
  public readonly staticDraws: StaticDrawBuffers = new StaticDrawBuffers(this.retirement);
  /** The numbers and array layers of every surface a static batch's shared material draws. */
  public readonly surfaceTable: SurfaceTable = new SurfaceTable(this.retirement);
  /** The sun's shadow cascades, fitted every frame. */
  public readonly shadows: ShadowUniforms = new ShadowUniforms();
  /** How the trees sway, built each frame. */
  public readonly treeWind: TreeWindUniforms = new TreeWindUniforms();
  /** How the grass sways, built each frame beside the trees' wind. */
  public readonly grassWind: GrassWindUniforms = new GrassWindUniforms();
  /** The sky the frame draws behind the scene and the water reflects. */
  public readonly sky: SkyUniforms = new SkyUniforms();
  /** The clouds the frame draws over the sky. */
  public readonly clouds: CloudUniforms = new CloudUniforms();
  /** How the water moves. */
  public readonly water: WaterUniforms = new WaterUniforms();
  /** What the motion every G-buffer surface writes is measured with. */
  public readonly motion: MotionUniforms = new MotionUniforms();

  private farPlane: Nullable<number> = null;
  private isLit: boolean = true;

  /**
   * Frees the storage retired a frame ago, the static pools' growths among it. Once a frame, whatever it draws.
   *
   * @param renderer - The renderer that uploaded it.
   */
  public freeRetired(renderer: WebGPURenderer): void {
    this.retirement.free(renderer);
  }

  /** How far anything can be seen: to the weather's far plane while the frame is lit and fogged, without end otherwise. */
  public get viewDistance(): number {
    return this.isLit && this.farPlane !== null ? this.farPlane : Infinity;
  }

  /**
   * @param settings - The consumer's settings.
   */
  public configure(settings: IRendererSettings): void {
    this.settings.apply(settings);
    this.water.apply(settings.features.water);
    this.sky.setDrawn(settings.isSkyDrawn);
    this.isLit = settings.isLit;
  }

  /**
   * @param lighting - How the scene is lit.
   */
  public light(lighting: IRendererLighting): void {
    this.lighting.apply(toBaseLightingConstants(lighting));
    this.treeWind.take(lighting.trees);
    this.grassWind.take(lighting.grass);
    this.water.take(lighting);
    this.sky.take(lighting.sky);
    this.clouds.take(lighting.sky.clouds);
    this.farPlane = lighting.fog?.farPlane ?? null;
  }

  /**
   * @param camera - The camera about to draw, with its matrices current.
   */
  public follow(camera: PerspectiveCamera): void {
    this.camera.follow(camera);
    this.lighting.follow(camera);
  }

  public dispose(): void {
    this.lut.dispose();
  }
}
