import { Nullable } from "@xrf/types";
import { Data3DTexture, PerspectiveCamera } from "three/webgpu";

import { IRendererLighting } from "#/contract/renderer-lighting";
import { IRendererSettings } from "#/contract/renderer-settings";
import { toBaseLightingConstants } from "#/lighting/base-lighting";
import { BaseLightingUniforms } from "#/uniforms/base-lighting-uniforms";
import { CameraUniforms } from "#/uniforms/camera-uniforms";
import { createMaterialLutTexture } from "#/uniforms/material-lut-texture";
import { SettingsUniforms } from "#/uniforms/settings-uniforms";
import { ShadowUniforms } from "#/uniforms/shadow-uniforms";
import { StaticDrawBuffers } from "#/uniforms/static-draw-buffers";

/**
 * Everything the frame's shaders read besides the scene: uniforms updated in place, so no change recompiles a
 * material, the material lookup every lit shader samples, and the buffers static draws are placed by. The uniforms sit
 * in three's render group, refreshed once a render call rather than per object, which a static draw's replay relies on.
 */
export class RendererUniforms {
  public readonly camera: CameraUniforms = new CameraUniforms();
  public readonly lighting: BaseLightingUniforms = new BaseLightingUniforms();
  public readonly settings: SettingsUniforms = new SettingsUniforms();
  public readonly lut: Data3DTexture = createMaterialLutTexture();
  /** What every static draw is culled and placed by. */
  public readonly staticDraws: StaticDrawBuffers = new StaticDrawBuffers();
  /** The sun's shadow cascades, fitted every frame. */
  public readonly shadows: ShadowUniforms = new ShadowUniforms();

  private fogDistance: Nullable<number> = null;
  private isLit: boolean = true;

  /**
   * How far anything can be seen: to where the fog is total while the frame is lit and fogged, which is where the
   * engine puts its far plane, and without end otherwise.
   */
  public get viewDistance(): number {
    return this.isLit && this.fogDistance !== null ? this.fogDistance : Infinity;
  }

  /**
   * @param settings - The consumer's settings.
   */
  public configure(settings: IRendererSettings): void {
    this.settings.apply(settings);
    this.isLit = settings.isLit;
  }

  /**
   * @param lighting - How the scene is lit.
   */
  public light(lighting: IRendererLighting): void {
    this.lighting.apply(toBaseLightingConstants(lighting));
    this.fogDistance = lighting.fog?.distance ?? null;
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
