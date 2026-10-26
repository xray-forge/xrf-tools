import { listRenderNumberFieldPaths, TRenderFeaturePath } from "@/core/render/lib/settings/render-feature-path";
import { RENDER_FEATURE_SCHEMA } from "@/core/render/lib/settings/render-feature-schema";
import { IRenderNumberField } from "@/core/render/lib/settings/render-setting-field";

/**
 * The bounds a value is offered between, the step it moves by, and what a slider measures it by: its place among the
 * feature settings and the range xrf recommends, where the schema rates it.
 */
export interface IRenderLimits {
  min: number;
  max: number;
  step: number;
  referencePath?: TRenderFeaturePath;
  recommended?: readonly [number, number];
  wastedAbove?: number;
}

/** Every number field of the feature settings by its place among them. */
const FIELD_PATHS: Map<IRenderNumberField, TRenderFeaturePath> = listRenderNumberFieldPaths(RENDER_FEATURE_SCHEMA);

/**
 * @param field - What the renderer takes of a feature setting.
 * @param step - The step it is offered by.
 * @returns Its bounds as the renderer holds them, the step as offered, its place among the feature settings and the
 *   range the schema recommends.
 */
export function toRenderLimits(field: IRenderNumberField, step: number): IRenderLimits {
  return {
    max: field.max,
    min: field.min,
    recommended: field.recommended,
    referencePath: FIELD_PATHS.get(field),
    step,
    wastedAbove: field.wastedAbove,
  };
}
