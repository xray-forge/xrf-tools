import { IFrameCompileTargets } from "#/graph/frame-compile-targets";
import { IRendererScenePass } from "#/pass/renderer-scene-pass";

/** The frame as the compile lane sees it: where it draws now, and what lets a pass joining it be drawn. */
export interface ICompilingFrame {
  /** Read again at each step of a batch, since a configure between two changes it. */
  readonly compileTargets: IFrameCompileTargets;
  /**
   * @param pass - A pass joining the frame, compiled for everything the scene draws: drawn from the next frame, or
   *   nothing where it left meanwhile.
   */
  admit(pass: IRendererScenePass): void;
}
