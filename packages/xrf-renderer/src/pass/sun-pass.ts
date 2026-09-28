import { NodeMaterial, QuadMesh, RenderTarget } from "three/webgpu";

import { createQuadMaterial } from "#/pass/quad-material";
import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererPass } from "#/pass/renderer-pass";
import { RendererTargets } from "#/pass/renderer-targets";
import { toSunPassFragment } from "#/pass/sun-pass.tsl";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";

/**
 * The sun, accumulated as `accum_sun` does: `Ldynamic_color * plight_infinity(m, P, N, L)`, times its shadow.
 */
export class SunPass implements IRendererPass {
  public readonly name: string = "sun";

  private readonly targets: RendererTargets;
  private readonly material: NodeMaterial;
  private readonly quad: QuadMesh;

  public constructor(targets: RendererTargets, uniforms: RendererUniforms) {
    this.targets = targets;
    this.material = createQuadMaterial(
      toSunPassFragment(
        targets,
        uniforms,
        targets.shadows.map((target: RenderTarget) => target.depthTexture as NonNullable<typeof target.depthTexture>)
      )
    );
    this.quad = new QuadMesh(this.material);
  }

  public render({ renderer }: IRendererFrame): void {
    renderer.setClearColor(0x000000, 0);
    renderer.setRenderTarget(this.targets.light);
    renderer.clear(true, false, false);
    this.quad.render(renderer);
  }

  public dispose(): void {
    this.material.dispose();
  }
}
