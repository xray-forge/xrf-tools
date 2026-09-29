import { renderGroup, uniform } from "three/tsl";
import { UniformNode } from "three/webgpu";

import { ERendererEngine } from "#/contract/renderer-engine";

/** The engine the scene is drawn as, which every shader porting both engines' draws by. */
export class EngineUniforms {
  /** One on the extended engine, nought on vanilla. */
  public readonly extended: UniformNode<"float", number> = uniform(0).setGroup(renderGroup);

  /**
   * @param engine - The engine the lighting is drawn as.
   */
  public take(engine: ERendererEngine): void {
    this.extended.value = engine === ERendererEngine.EXTENDED ? 1 : 0;
  }
}
