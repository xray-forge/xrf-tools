import { Fn, getViewPosition, screenUV, texture } from "three/tsl";
import { Data3DTexture, NodeMaterial, QuadMesh } from "three/webgpu";

import { BaseLightingUniforms } from "#/graph/base-lighting-uniforms";
import { toSunLight } from "#/graph/base-lighting.tsl";
import { CameraUniforms } from "#/graph/camera-uniforms";
import { decodeOctahedral } from "#/graph/octahedral-normal.tsl";
import { IRendererFrame } from "#/graph/renderer-frame";
import { RendererTargets } from "#/graph/renderer-targets";
import { createQuadMaterial } from "#/pass/quad-material";
import { IRendererPass } from "#/pass/renderer-pass";

/**
 * The sun, accumulated as `accum_sun` does: `Ldynamic_color * plight_infinity(m, P, N, L)`, unshadowed for now.
 */
export class SunPass implements IRendererPass {
  public readonly name: string = "sun";

  private readonly material: NodeMaterial;
  private readonly quad: QuadMesh;

  public constructor(
    targets: RendererTargets,
    camera: CameraUniforms,
    lighting: BaseLightingUniforms,
    lut: Data3DTexture
  ) {
    // todo: multiply by the sun shadow once the cached cascades of decision 16 exist.
    const fragment = Fn(() => {
      const depth = texture(targets.depth, screenUV).x;

      return toSunLight(
        {
          normal: decodeOctahedral(texture(targets.normal, screenUV).xy),
          position: getViewPosition(screenUV, depth, camera.projectionInverse),
          slice: texture(targets.surface, screenUV).z,
        },
        lighting,
        lut
      );
    })();

    this.material = createQuadMaterial(fragment);
    this.quad = new QuadMesh(this.material);
  }

  public render({ renderer, targets }: IRendererFrame): void {
    renderer.setClearColor(0x000000, 0);
    renderer.setRenderTarget(targets.light);
    renderer.clear(true, false, false);
    this.quad.render(renderer);
  }

  public dispose(): void {
    this.material.dispose();
  }
}
