import { ERendererBumpPlane } from "#/contract/renderer-bump-plane";
import { ERendererDebugView } from "#/contract/renderer-debug-view";

/** What a capture draws. */
export enum ERendererCaptureSource {
  /** The frame the attached view last drew, or one of its targets. */
  FRAME = "frame",
  /** One plane of a bump pair, face on and texel for texel, with no view needed. */
  BUMP_PLANE = "bumpPlane",
}

/** A capture's subject, with what it needs. */
export type TRendererCaptureSource =
  | { kind: ERendererCaptureSource.FRAME; view: ERendererDebugView }
  | {
      kind: ERendererCaptureSource.BUMP_PLANE;
      plane: ERendererBumpPlane;
      /** The key the pair's first half was put under. */
      bump: string;
      /** The key its companion was put under. */
      companion: string;
      width: number;
      height: number;
    };
