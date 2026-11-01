import { formatNumber } from "@xrf/format";

import { IRenderLimits } from "@/core/render/lib/features/render-limits";
import {
  ERenderSettingControl,
  IRenderSliderSetting,
  TRenderSettings,
} from "@/core/render/lib/features/render-setting";
import { TRenderCameraSettings } from "@/core/render/lib/settings/choice/render-feature-settings";

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

/**
 * @param effect - The camera effect the value shapes, which it changes nothing without.
 * @param label - What it is called.
 * @param description - What it does.
 * @param limits - Between what it is offered.
 * @param format - How it reads.
 * @returns One of a camera effect's values, inert while the effect is off.
 */
function toCameraEffectSlider(
  effect: keyof TRenderCameraSettings,
  label: string,
  description: string,
  limits: IRenderLimits,
  format: (value: number) => string
): IRenderSliderSetting<TRenderCameraSettings> {
  return {
    control: ERenderSettingControl.SLIDER,
    description,
    format,
    isInert: (camera: TRenderCameraSettings) => !camera[effect],
    label,
    limits,
  };
}

/**
 * How each camera setting is set, in the settings and the Camera effects popover alike. Each value is offered between
 * bounds narrower than the renderer takes, where a camera is used: a shutter open longer than the frame or a blur past
 * a tenth of the frame smears a turn into a wash; the lens from f/1.4 to f/16, focused out to 200 m, past which the far
 * field is all in focus at any aperture a level's view uses; sensors from a compact's to medium format.
 */
export const RENDER_CAMERA_SETTINGS: TRenderSettings<TRenderCameraSettings> = {
  aberrationStrength: {
    ...toCameraEffectSlider(
      "isAberrated",
      "Aberration strength",
      "How far apart the colours split at the corners.",
      { max: 1, min: 0, step: 0.05 },
      formatCameraStrength
    ),
    entryLabel: "Strength",
  },
  fStop: toCameraEffectSlider(
    "isDefocused",
    "Aperture",
    "The lens's f-number: lower opens it wider and keeps less in focus.",
    { max: 16, min: 1.4, step: 0.1 },
    formatCameraFStop
  ),
  focusDistance: toCameraEffectSlider(
    "isDefocused",
    "Focus",
    "How far from the camera the lens is focused.",
    { max: 200, min: 0.5, step: 0.5 },
    formatCameraFocusDistance
  ),
  grainSize: {
    ...toCameraEffectSlider(
      "isGrained",
      "Grain size",
      "How many pixels across a grain is.",
      { max: 3, min: 1, step: 0.1 },
      formatCameraGrainSize
    ),
    entryLabel: "Size",
  },
  grainStrength: {
    ...toCameraEffectSlider(
      "isGrained",
      "Grain strength",
      "How strongly the grain shows.",
      { max: 1, min: 0, step: 0.05 },
      formatCameraStrength
    ),
    entryLabel: "Strength",
  },
  isAberrated: {
    control: ERenderSettingControl.CHECKBOX,
    description: "Splits red and blue apart towards the frame's edges, as a lens's glass bends them unequally.",
    label: "Chromatic aberration",
  },
  isDefocused: {
    control: ERenderSettingControl.CHECKBOX,
    description:
      "Blurs what stands nearer or farther than the focus, as a lens of the view's field of view does. A pick in the " +
      "viewer sets the focus.",
    label: "Depth of field",
  },
  isGrained: {
    control: ERenderSettingControl.CHECKBOX,
    description: "Lays a fine grain over the frame, different every frame, strongest in the mid tones.",
    label: "Film grain",
  },
  isMotionBlurred: {
    control: ERenderSettingControl.CHECKBOX,
    description: "Blurs what moves across the frame, and the frame as the camera turns, along its motion.",
    label: "Motion blur",
  },
  isVignetted: {
    control: ERenderSettingControl.CHECKBOX,
    description: "Dims the frame's corners, as light reaching the film at an angle falls off through a lens.",
    label: "Vignette",
  },
  motionLimit: toCameraEffectSlider(
    "isMotionBlurred",
    "Longest blur",
    "How far a blur reaches at most, so a fast turn smears no further.",
    { max: 0.1, min: 0.01, step: 0.005 },
    formatCameraMotionLimit
  ),
  sensorHeight: toCameraEffectSlider(
    "isDefocused",
    "Sensor",
    "The sensor's height: larger blurs more at the same view and aperture. 24 mm is full frame.",
    { max: 44, min: 8, step: 1 },
    formatCameraSensorHeight
  ),
  shutter: toCameraEffectSlider(
    "isMotionBlurred",
    "Shutter",
    "How long the shutter stays open each frame, as a film camera's angle: 180° is film's look.",
    { max: 1, min: 0.05, step: 0.05 },
    formatCameraShutter
  ),
  vignetteStrength: {
    ...toCameraEffectSlider(
      "isVignetted",
      "Vignette strength",
      "How much of the lens's falloff the corners show.",
      { max: 1, min: 0, step: 0.05 },
      formatCameraStrength
    ),
    entryLabel: "Strength",
  },
};
