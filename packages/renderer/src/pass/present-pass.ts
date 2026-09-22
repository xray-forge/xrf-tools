import { Nullable } from "@xrf/types";
import { float, Fn, getViewPosition, log, screenUV, select, texture, vec3, vec4 } from "three/tsl";
import { Node, NodeMaterial, QuadMesh, RenderTarget, WebGPURenderer } from "three/webgpu";

import { ERendererDebugView } from "#/contract/renderer-settings";
import { CameraUniforms } from "#/graph/camera-uniforms";
import { decodeOctahedral } from "#/graph/octahedral-normal.tsl";
import { IRendererFrame } from "#/graph/renderer-frame";
import { RendererTargets } from "#/graph/renderer-targets";
import { createQuadMaterial } from "#/pass/quad-material";
import { IRendererPass } from "#/pass/renderer-pass";

/**
 * Puts the chosen picture on the canvas: the frame, or one target shown raw.
 */
export class PresentPass implements IRendererPass {
  public readonly name: string = "present";

  private readonly quad: QuadMesh = new QuadMesh();
  private readonly materials: Map<ERendererDebugView, NodeMaterial> = new Map();
  private readonly targets: RendererTargets;
  private readonly camera: CameraUniforms;

  public constructor(targets: RendererTargets, camera: CameraUniforms) {
    this.targets = targets;
    this.camera = camera;
  }

  public render({ renderer, settings }: IRendererFrame): void {
    this.draw(renderer, settings.debugView, null);
  }

  /**
   * @param renderer - The renderer drawing.
   * @param view - The picture wanted.
   * @param target - Where it goes: the canvas when null.
   */
  public draw(renderer: WebGPURenderer, view: ERendererDebugView, target: Nullable<RenderTarget>): void {
    let material: Nullable<NodeMaterial> = this.materials.get(view) ?? null;

    if (!material) {
      material = createQuadMaterial(this.describe(view));
      this.materials.set(view, material);
    }

    this.quad.material = material;
    renderer.setRenderTarget(target);
    this.quad.render(renderer);
  }

  public dispose(): void {
    this.materials.forEach((material: NodeMaterial) => material.dispose());
    this.materials.clear();
  }

  private describe(view: ERendererDebugView): Node {
    const { albedo, normal, surface, depth, light, scene } = this.targets;

    if (view === ERendererDebugView.FINAL) {
      return texture(scene.texture, screenUV);
    }

    return Fn(() => {
      const stored = texture(depth, screenUV).x;
      // A target shown raw is transparent where nothing was drawn, so it is never mistaken for a cleared value.
      const coverage = select(stored.lessThan(1), float(1), float(0));

      function shown(value: Node<"vec3">): Node<"vec4"> {
        return vec4(value, coverage);
      }

      switch (view) {
        case ERendererDebugView.ALBEDO:
          return shown(texture(albedo, screenUV).xyz);

        case ERendererDebugView.GLOSS:
          return shown(texture(albedo, screenUV).www);

        case ERendererDebugView.NORMAL:
          return shown(decodeOctahedral(texture(normal, screenUV).xy).mul(0.5).add(0.5));

        case ERendererDebugView.HEMI:
          return shown(texture(surface, screenUV).xxx);

        case ERendererDebugView.SUN:
          return shown(texture(surface, screenUV).yyy);

        case ERendererDebugView.MATERIAL:
          return shown(texture(surface, screenUV).zzz);

        case ERendererDebugView.DEPTH: {
          // Logarithmic, so a metre up close and a kilometre away both read.
          const distance = getViewPosition(screenUV, stored, this.camera.projectionInverse).z.negate();

          return shown(vec3(log(distance.add(1)).div(log(this.camera.far.add(1)))));
        }

        case ERendererDebugView.LIGHT:
          return shown(texture(light.texture, screenUV).xyz);
      }
    })();
  }
}
