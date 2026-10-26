import { Nullable } from "@xrf/types";

import {
  ERenderSettingKind,
  IRenderNumberField,
  isRenderSettingField,
} from "@/core/render/lib/settings/render-setting-field";

/** A setting's place among the feature settings: the keys leading to it, `["reflections", "steps"]`. */
export type TRenderFeaturePath = ReadonlyArray<string>;

/**
 * @param schema - A schema node, such as the feature settings' whole schema.
 * @returns Each number field under it by its place.
 */
export function listRenderNumberFieldPaths(
  schema: object,
  path: TRenderFeaturePath = [],
  found: Map<IRenderNumberField, TRenderFeaturePath> = new Map()
): Map<IRenderNumberField, TRenderFeaturePath> {
  if (isRenderSettingField(schema)) {
    if (schema.kind === ERenderSettingKind.NUMBER) {
      found.set(schema, path);
    }

    return found;
  }

  for (const [key, child] of Object.entries(schema)) {
    if (typeof child === "object" && child !== null) {
      listRenderNumberFieldPaths(child, [...path, key], found);
    }
  }

  return found;
}

/**
 * @param settings - Feature settings, or any part of them.
 * @param path - A place in them.
 * @returns The number standing there, or none where something else or nothing does.
 */
export function readRenderFeatureNumber(settings: object, path: TRenderFeaturePath): Nullable<number> {
  let node: unknown = settings;

  for (const key of path) {
    node = typeof node === "object" && node !== null ? (node as Record<string, unknown>)[key] : undefined;
  }

  return typeof node === "number" && Number.isFinite(node) ? node : null;
}
