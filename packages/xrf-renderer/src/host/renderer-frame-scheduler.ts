/** Schedules a callback for the next frame, as `requestAnimationFrame` does. */
export type TRendererFrameScheduler = (callback: (now: number) => void) => number;
