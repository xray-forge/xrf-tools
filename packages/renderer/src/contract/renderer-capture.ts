import { ERendererDebugView } from "#/contract/renderer-settings";

/**
 * One plane of a bump pair, as stored or as the engine reconstructs it.
 */
export enum ERendererBumpPlane {
  /** `normal.gloss` as stored: the normal reversed into green, blue and alpha, gloss in red. */
  BUMP = "bump",
  /** `normal_error.height` as stored: the quantisation error of the normal in rgb, height in alpha. */
  COMPANION = "companion",
  /** The tangent space normal the engine reconstructs, mapped into the unit range. */
  NORMAL = "normal",
  /** Gloss, the bump's red squared. */
  GLOSS = "gloss",
  /** Height as the companion stores it in alpha. */
  HEIGHT = "height",
}

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
