import { RENDER_FEATURE_SCHEMA } from "@/core/render/lib/settings/render-feature-schema";
import {
  ERenderSettingKind,
  IRenderNumberField,
  isRenderSettingField,
} from "@/core/render/lib/settings/render-setting-field";

/**
 * The bounds a value is offered between, the step it moves by, and what a slider measures it by: its place among the
 * feature settings and the range xrf recommends, where the schema rates it.
 */
export interface IRenderLimits {
  min: number;
  max: number;
  step: number;
  referencePath?: ReadonlyArray<string>;
  recommended?: readonly [number, number];
}

/** Every number field of the feature settings by its place among them, found once. */
let fieldPaths: Map<IRenderNumberField, ReadonlyArray<string>> | null = null;

/**
 * @param field - What the renderer takes of a feature setting.
 * @param step - The step it is offered by.
 * @returns Its bounds as the renderer holds them, the step as offered, its place among the feature settings, which a
 *   slider's reference is read at, and the range the schema recommends.
 */
export function toRenderLimits(field: IRenderNumberField, step: number): IRenderLimits {
  fieldPaths ??= listFieldPaths(RENDER_FEATURE_SCHEMA, []);

  return {
    max: field.max,
    min: field.min,
    recommended: field.recommended,
    referencePath: fieldPaths.get(field),
    step,
  };
}

/** Each number field under a schema node by the keys leading to it. */
function listFieldPaths(
  node: object,
  path: ReadonlyArray<string>,
  found: Map<IRenderNumberField, ReadonlyArray<string>> = new Map()
): Map<IRenderNumberField, ReadonlyArray<string>> {
  if (isRenderSettingField(node)) {
    if (node.kind === ERenderSettingKind.NUMBER) {
      found.set(node, path);
    }

    return found;
  }

  for (const [key, child] of Object.entries(node)) {
    if (typeof child === "object" && child !== null) {
      listFieldPaths(child, [...path, key], found);
    }
  }

  return found;
}
