import { dot, Fn, getViewPosition, normalize, screenUV, texture, texture3D, vec3, vec4 } from "three/tsl";
import { Data3DTexture, NodeMaterial, QuadMesh } from "three/webgpu";

import { BaseLightingUniforms } from "#/graph/base-lighting-uniforms";
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
      const position = getViewPosition(screenUV, depth, camera.projectionInverse);
      const normal = decodeOctahedral(texture(targets.normal, screenUV).xy);
      const slice = texture(targets.surface, screenUV).z;
      // `plight_infinity`: L towards the light, V towards the eye, H halfway.
      const toLight = lighting.sunDirectionView.negate();
      const half = normalize(toLight.add(normalize(position).negate()));
      const sample = texture3D(lut, vec3(dot(toLight, normal), dot(half, normal), slice));

      return vec4(lighting.sunColor.mul(sample.x), lighting.sunSpecular.mul(sample.y));
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
