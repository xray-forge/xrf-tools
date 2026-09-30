import {
  CustomBlending,
  HalfFloatType,
  NearestFilter,
  NodeMaterial,
  OneFactor,
  RenderTarget,
  SrcColorFactor,
  ZeroFactor,
} from "three/webgpu";

import { FullScreenDraw } from "#/pass/full-screen-draw";
import { createQuadMaterial } from "#/pass/quad-material";
import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererPass } from "#/pass/renderer-pass";
import { IRendererPipelines } from "#/pass/renderer-pipelines";
import { RendererTargets } from "#/pass/renderer-targets";
import { toWetGlossFragment, toWetNormalFragment, toWetPatchFragment } from "#/shader/wet.tsl";
import { RainUniforms } from "#/uniforms/rain-uniforms";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";

/**
 * Rain on the G-buffer before any light, as `draw_rain` wets it (`r3_rendertarget_draw_rain.cpp`): where the rain
 * reaches near the camera, its normals patched into a target of their own, then written back, then the albedo darkened
 * and the gloss raised by how wet it is. Draws nothing while it does not rain.
 */
export class WetPass implements IRendererPass {
  public readonly name: string = "wet";

  private readonly targets: RendererTargets;
  private readonly rain: RainUniforms;
  /** The patched normal in colour, the wetness in alpha: `rt_Accumulator` as the engine borrows it. */
  private readonly patched: RenderTarget = new RenderTarget(1, 1, { depthBuffer: false, type: HalfFloatType });
  /** The normals patched, written back, then the albedo and gloss wetted. */
  private readonly draws: readonly [FullScreenDraw, FullScreenDraw, FullScreenDraw];

  /**
   * @param targets - The frame's targets, whose G-buffer is wetted.
   * @param uniforms - What the frame's shaders read, the rain's cover and the wet surfaces among them.
   */
  public constructor(targets: RendererTargets, uniforms: RendererUniforms) {
    this.targets = targets;
    this.rain = uniforms.rain;
    this.patched.texture.name = "wet-patched";
    this.patched.texture.minFilter = this.patched.texture.magFilter = NearestFilter;

    const gloss: NodeMaterial = createQuadMaterial(
      toWetGlossFragment({
        depth: targets.depth,
        engine: uniforms.engine,
        patched: this.patched.texture,
        wet: uniforms.wet,
      })
    );

    // `blend(zero, srccolor)` on colour and `(one, one)` on alpha: the albedo multiplied, the gloss added.
    gloss.blending = CustomBlending;
    gloss.blendSrc = ZeroFactor;
    gloss.blendDst = SrcColorFactor;
    gloss.blendSrcAlpha = OneFactor;
    gloss.blendDstAlpha = OneFactor;
    this.draws = [
      new FullScreenDraw(
        createQuadMaterial(
          toWetPatchFragment({
            camera: uniforms.camera,
            engine: uniforms.engine,
            rain: uniforms.rain,
            textures: targets,
            wet: uniforms.wet,
          })
        ),
        this.patched
      ),
      new FullScreenDraw(
        createQuadMaterial(toWetNormalFragment(this.patched.texture, targets.depth)),
        targets.wetNormal
      ),
      new FullScreenDraw(gloss, targets.wetAlbedo),
    ];
  }

  /** Named whether or not it rains, so the first rain draws at once. */
  public listPipelines(pipelines: IRendererPipelines): void {
    for (const draw of this.draws) {
      pipelines.draw(draw);
    }
  }

  public render({ renderer }: IRendererFrame): void {
    if (!this.rain.isFalling) {
      return;
    }

    const { width, height } = this.targets.gbuffer;

    if (this.patched.width !== width || this.patched.height !== height) {
      this.patched.setSize(width, height);
    }

    for (const draw of this.draws) {
      draw.render(renderer);
    }
  }

  public dispose(): void {
    this.patched.dispose();
    this.draws.forEach((draw: FullScreenDraw) => draw.dispose());
  }
}
