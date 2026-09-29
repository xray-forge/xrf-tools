import { TRendererVector } from "#/contract/renderer-vector";

/**
 * One of a striking bolt's glows, where it stands this frame.
 */
export interface IRendererThunderboltGlow {
  /** In engine space. */
  position: TRendererVector;
  /** Half its width and half its height, in metres. */
  extent: readonly [number, number];
  /** What its colour and alpha are both scaled by. */
  opacity: number;
}
