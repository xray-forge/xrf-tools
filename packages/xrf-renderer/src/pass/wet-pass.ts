import {
  CustomBlending,
  HalfFloatType,
  NearestFilter,
  NodeMaterial,
  OneFactor,
  QuadMesh,
  RenderTarget,
  SrcColorFactor,
  ZeroFactor,
} from "three/webgpu";

import { createQuadMaterial } from "#/pass/quad-material";
import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererPass } from "#/pass/renderer-pass";
import { RendererTargets } from "#/pass/renderer-targets";
import { toWetGlossFragment, toWetNormalFragment, toWetPatchFragment } from "#/shader/wet.tsl";
import { RainUniforms } from "#/uniforms/rain-uniforms";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";
import { WetUniforms } from "#/uniforms/wet-uniforms";

/**
 * Rain on the G-buffer before any light, as `draw_rain` wets it (`r3_rendertarget_draw_rain.cpp`): where the rain
 * reaches near the camera, its normals patched into a target of their own, then written back, then the albedo darkened
 * and the gloss raised by how wet it is. Draws nothing while it does not rain.
 */
export class WetPass implements IRendererPass {
  public readonly name: string = "wet";

  private readonly targets: RendererTargets;
  private readonly rain: RainUniforms;
  private readonly wet: WetUniforms;
  /** The patched normal in colour, the wetness in alpha: `rt_Accumulator` as the engine borrows it. */
  private readonly patched: RenderTarget = new RenderTarget(1, 1, { depthBuffer: false, type: HalfFloatType });
  private readonly materials: readonly [NodeMaterial, NodeMaterial, NodeMaterial];
  private readonly quad: QuadMesh = new QuadMesh();

  /**
   * @param targets - The frame's targets, whose G-buffer is wetted.
   * @param uniforms - What the frame's shaders read, the rain's cover and the wet surfaces among them.
   */
  public constructor(targets: RendererTargets, uniforms: RendererUniforms) {
    this.targets = targets;
    this.rain = uniforms.rain;
    this.wet = uniforms.wet;
    this.patched.texture.name = "wet-patched";
    this.patched.texture.minFilter = this.patched.texture.magFilter = NearestFilter;

    const gloss: NodeMaterial = createQuadMaterial(
      toWetGlossFragment(this.patched.texture, targets.depth, uniforms.wet)
    );

    // `blend(zero, srccolor)` on colour and `(one, one)` on alpha: the albedo multiplied, the gloss added.
    gloss.blending = CustomBlending;
    gloss.blendSrc = ZeroFactor;
    gloss.blendDst = SrcColorFactor;
    gloss.blendSrcAlpha = OneFactor;
    gloss.blendDstAlpha = OneFactor;
    this.materials = [
      createQuadMaterial(
        toWetPatchFragment({ camera: uniforms.camera, rain: uniforms.rain, textures: targets, wet: uniforms.wet })
      ),
      createQuadMaterial(toWetNormalFragment(this.patched.texture, targets.depth)),
      gloss,
    ];
  }

  public render({ renderer }: IRendererFrame): void {
    if (!this.rain.isFalling || !this.wet.isRaining) {
      return;
    }

    const { width, height } = this.targets.gbuffer;

    if (this.patched.width !== width || this.patched.height !== height) {
      this.patched.setSize(width, height);
    }

    [this.patched, this.targets.wetNormal, this.targets.wetAlbedo].forEach((target: RenderTarget, index: number) => {
      renderer.setRenderTarget(target);
      this.quad.material = this.materials[index];
      this.quad.render(renderer);
    });
  }

  public dispose(): void {
    this.patched.dispose();
    this.materials.forEach((material: NodeMaterial) => material.dispose());
  }
}
