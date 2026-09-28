/**
 * A motion baked to model space bone transforms, every frame.
 */
export interface IRendererMotion {
  /** Frame major: each frame is every bone's transform in turn. */
  transforms: Float32Array;
  /** Floats one bone takes in `transforms`, the bind layout's twelve at least. */
  floatsPerBone: number;
}

/**
 * What of a motion moves between threads: its transforms, never copied.
 *
 * @param motion - The motion about to be posted.
 * @returns Its buffer.
 */
export function listRendererMotionTransfers(motion: IRendererMotion): Array<Transferable> {
  return [motion.transforms.buffer];
}
