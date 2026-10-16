import { IRenderNumberField } from "@/core/render/lib/settings/render-setting-field";

/** The bounds a value is offered between, and the step it moves by. */
export interface IRenderLimits {
  min: number;
  max: number;
  step: number;
}

/**
 * @param field - What the renderer takes of a setting.
 * @param step - The step it is offered by.
 * @returns Its bounds as the renderer holds them, the step as offered.
 */
export function toRenderLimits(field: IRenderNumberField, step: number): IRenderLimits {
  return { max: field.max, min: field.min, step };
}
