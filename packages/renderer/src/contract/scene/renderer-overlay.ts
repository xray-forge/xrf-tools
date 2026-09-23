import { TRendererColor } from "#/contract/renderer-lighting";

/**
 * What kind of helper an overlay draws.
 */
export enum ERendererOverlay {
  /** Line segments, two points each, coloured per vertex. */
  LINES = "lines",
  /** Points of one colour, a fixed number of pixels across wherever they are. */
  POINTS = "points",
  /** A skeleton's bones as segments, following its pose. */
  SKELETON = "skeleton",
  /** A disc in the sky where the light comes from, following the camera and the lighting. */
  SUN = "sun",
}

/**
 * A helper drawn over the frame, unlit, as the raw colours it names.
 */
export type TRendererOverlay =
  | {
      kind: ERendererOverlay.LINES;
      /** Three floats a vertex, two vertices a segment. */
      positions: Float32Array;
      /** Three floats a vertex. */
      colors: Float32Array;
      /** Whether what stands in front hides it. */
      isDepthTested: boolean;
    }
  | {
      kind: ERendererOverlay.POINTS;
      positions: Float32Array;
      color: TRendererColor;
      /** Width in device pixels. */
      size: number;
      isDepthTested: boolean;
    }
  | {
      kind: ERendererOverlay.SKELETON;
      /** The key the skeleton was put under. */
      skeleton: string;
      color: TRendererColor;
      isDepthTested: boolean;
    }
  | {
      kind: ERendererOverlay.SUN;
      color: TRendererColor;
      /** Width in device pixels. */
      size: number;
    };

/**
 * What of an overlay moves between threads: its arrays, never copied.
 *
 * @param overlay - The overlay about to be posted.
 * @returns Its buffers.
 */
export function listRendererOverlayTransfers(overlay: TRendererOverlay): Array<Transferable> {
  switch (overlay.kind) {
    case ERendererOverlay.LINES:
      return [overlay.positions.buffer, overlay.colors.buffer];

    case ERendererOverlay.POINTS:
      return [overlay.positions.buffer];

    case ERendererOverlay.SKELETON:
    case ERendererOverlay.SUN:
      return [];
  }
}
