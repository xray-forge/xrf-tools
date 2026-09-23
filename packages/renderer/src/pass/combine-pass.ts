import { Discard, float, Fn, getViewPosition, If, mix, screenUV, select, texture, vec4 } from "three/tsl";
import { Color, Data3DTexture, LinearSRGBColorSpace, NodeMaterial, QuadMesh } from "three/webgpu";

import { BaseLightingUniforms } from "#/graph/base-lighting-uniforms";
import { toBaseColor, toFinishedColor } from "#/graph/base-lighting.tsl";
import { CameraUniforms } from "#/graph/camera-uniforms";
import { decodeOctahedral } from "#/graph/octahedral-normal.tsl";
import { IRendererFrame } from "#/graph/renderer-frame";
import { RendererTargets } from "#/graph/renderer-targets";
import { SettingsUniforms } from "#/graph/settings-uniforms";
import { createQuadMaterial } from "#/pass/quad-material";
import { IRendererPass } from "#/pass/renderer-pass";

/**
 * `combine_1` and the tonemap of `combine_2`: the hemisphere model, the lights, fog, and the engine's Reinhard curve.
 */
export class CombinePass implements IRendererPass {
  public readonly name: string = "combine";

  private readonly material: NodeMaterial;
  private readonly quad: QuadMesh;
  private readonly backdrop: Color = new Color();

  public constructor(
    targets: RendererTargets,
    camera: CameraUniforms,
    lighting: BaseLightingUniforms,
    settings: SettingsUniforms,
    lut: Data3DTexture
  ) {
    const fragment = Fn(() => {
      const depth = texture(targets.depth, screenUV).x;

      // Where nothing was drawn the backdrop the target was cleared to shows.
      If(depth.greaterThanEqual(1), () => {
        Discard();
      });

      const albedo = texture(targets.albedo, screenUV);
      const surface = texture(targets.surface, screenUV);

      const point = {
        normal: decodeOctahedral(texture(targets.normal, screenUV).xy),
        position: getViewPosition(screenUV, depth, camera.projectionInverse),
        slice: surface.z,
      };

      const light = texture(targets.light.texture, screenUV);
      const hemi = mix(float(1), surface.x, settings.hemiStrength);
      const color = toBaseColor(albedo.xyz, albedo.w, light, hemi, point, lighting, camera, lut);
      const lit = toFinishedColor(color, point.position, lighting);

      // Unlit, the frame is the raw albedo: the file as it reads, with nothing the lighting model adds.
      return vec4(select(settings.lit.greaterThan(0.5), lit, albedo.xyz), 1);
    })();

    this.material = createQuadMaterial(fragment);
    this.quad = new QuadMesh(this.material);
  }

  public render({ renderer, targets, settings }: IRendererFrame): void {
    // The hex is bytes the page shows, so it reaches the canvas as written rather than decoded from srgb.
    this.backdrop.setHex(settings.backdrop ?? 0, LinearSRGBColorSpace);
    renderer.setClearColor(this.backdrop, settings.backdrop === null ? 0 : 1);
    renderer.setRenderTarget(targets.scene);
    renderer.clear(true, false, false);
    this.quad.render(renderer);
  }

  public dispose(): void {
    this.material.dispose();
  }
}
