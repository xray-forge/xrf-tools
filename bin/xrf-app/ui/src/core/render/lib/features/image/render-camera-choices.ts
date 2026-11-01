import { formatNumber } from "@xrf/format";

import { IRenderLimits } from "@/core/render/lib/features/render-limits";

/**
 * The bounds each camera value is offered between: narrower than the renderer takes, where a camera is used. A
 * shutter open longer than the frame or a blur past a tenth of the frame smears a turn into a wash; the lens from f/1.4
 * to f/16, focused out to 200 m, past which the far field is all in focus at any aperture a level's view uses; sensors
 * from a compact's to medium format.
 */
export const RENDER_CAMERA_LIMITS = {
  aberrationStrength: { max: 1, min: 0, step: 0.05 },
  fStop: { max: 16, min: 1.4, step: 0.1 },
  focusDistance: { max: 200, min: 0.5, step: 0.5 },
  grainSize: { max: 3, min: 1, step: 0.1 },
  grainStrength: { max: 1, min: 0, step: 0.05 },
  motionLimit: { max: 0.1, min: 0.01, step: 0.005 },
  sensorHeight: { max: 44, min: 8, step: 1 },
  shutter: { max: 1, min: 0.05, step: 0.05 },
  vignetteStrength: { max: 1, min: 0, step: 0.05 },
} as const satisfies Record<string, IRenderLimits>;

/** @returns The shutter's share of a frame as a film camera's shutter angle. */
export function formatCameraShutter(shutter: number): string {
  return `${formatNumber(shutter * 360, 0)}°`;
}

/** @returns The longest blur as a share of the frame's height. */
export function formatCameraMotionLimit(limit: number): string {
  return `${formatNumber(limit * 100, 1)}% of the height`;
}

/** @returns The aperture as an f-number. */
export function formatCameraFStop(fStop: number): string {
  return `f/${formatNumber(fStop, 1)}`;
}

/** @returns The focus's distance from the camera. */
export function formatCameraFocusDistance(distance: number): string {
  return `${formatNumber(distance, distance < 10 ? 1 : 0)} m`;
}

/** @returns The sensor's height. */
export function formatCameraSensorHeight(height: number): string {
  return `${formatNumber(height, 0)} mm`;
}

/** @returns A strength from none to all, as a percentage. */
export function formatCameraStrength(strength: number): string {
  return `${formatNumber(strength * 100, 0)}%`;
}

/** @returns A grain's size across. */
export function formatCameraGrainSize(size: number): string {
  return `${formatNumber(size, 1)} px`;
}
