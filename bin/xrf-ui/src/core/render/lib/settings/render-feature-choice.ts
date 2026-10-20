import { isRecord } from "@xrf/types";

import { IRenderFeatureOverrides } from "@/core/render/lib/settings/render-feature-overrides";
import { RENDER_FEATURE_SCHEMA } from "@/core/render/lib/settings/render-feature-schema";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/render-feature-settings";
import { ERenderPreset, RENDER_PRESETS } from "@/core/render/lib/settings/render-preset";
import {
  isRenderSettingField,
  isSameRenderSetting,
  toRenderSettingValue,
  TRenderSettingField,
} from "@/core/render/lib/settings/render-setting-field";

/** A preset and what was changed on top of it, which is what a consumer stores. */
export interface IRenderFeatureChoice {
  preset: ERenderPreset;
  overrides: IRenderFeatureOverrides;
}

export const DEFAULT_RENDER_FEATURE_CHOICE: IRenderFeatureChoice = {
  overrides: {},
  preset: ERenderPreset.BASE,
};

/** A node of the schema: one setting's field, or a group of nodes by key. */
type TSchemaNode = TRenderSettingField | { readonly [key: string]: TSchemaNode };

/** A group of values by key, as a group of settings or of their overrides is. */
type TValues = Record<string, unknown>;

const SCHEMA: TSchemaNode = RENDER_FEATURE_SCHEMA as unknown as TSchemaNode;

/**
 * @param stored - What a consumer stored for its choice, parsed from wherever it keeps it.
 * @returns The choice, every value the features do not take dropped and every number held to its bounds; the default
 *   for nothing usable.
 */
export function toRenderFeatureChoice(stored: unknown): IRenderFeatureChoice {
  if (!isRecord(stored)) {
    return DEFAULT_RENDER_FEATURE_CHOICE;
  }

  return {
    overrides: (parse(SCHEMA, stored.overrides) ?? {}) as IRenderFeatureOverrides,
    preset:
      Object.values(ERenderPreset).find((preset: ERenderPreset) => preset === stored.preset) ??
      DEFAULT_RENDER_FEATURE_CHOICE.preset,
  };
}

/**
 * @param choice - A preset and what was changed on top of it.
 * @returns Every feature as the choice sets it.
 */
export function resolveRenderFeatures(choice: IRenderFeatureChoice): IRenderFeatureSettings {
  return resolve(SCHEMA, RENDER_PRESETS[choice.preset], choice.overrides) as IRenderFeatureSettings;
}

/**
 * @param choice - A preset and what was changed on top of it.
 * @returns Whether the features differ from the preset's, which a settings view shows as custom.
 */
export function isRenderFeatureChoiceCustom(choice: IRenderFeatureChoice): boolean {
  return !isSame(SCHEMA, resolveRenderFeatures(choice), RENDER_PRESETS[choice.preset]);
}

/**
 * @param features - Every feature as a choice resolves it.
 * @param preset - The preset that choice stands on.
 * @param group - One feature group.
 * @returns Whether that group differs from the preset's, which a settings view marks where the group is set.
 */
export function isRenderFeatureCustom(
  features: IRenderFeatureSettings,
  preset: ERenderPreset,
  group: keyof IRenderFeatureSettings
): boolean {
  const node: TSchemaNode = (SCHEMA as { readonly [key: string]: TSchemaNode })[group];

  return !isSame(node, features[group], RENDER_PRESETS[preset][group]);
}

/**
 * @param overrides - What was changed on top of a preset.
 * @param changes - What changes now, setting by setting.
 * @returns The two together, each change in place of what it changes.
 */
export function mergeRenderFeatureOverrides(
  overrides: IRenderFeatureOverrides,
  changes: IRenderFeatureOverrides
): IRenderFeatureOverrides {
  return (merge(SCHEMA, overrides, changes) ?? {}) as IRenderFeatureOverrides;
}

/**
 * @param sent - Settings as a consumer gave or stored them.
 * @param kept - What reading them against the schema kept of them.
 * @returns The dotted path of every setting given that was not kept as given: a key the schema does not know, or a
 *   value it does not take or bounds to another.
 */
export function listUnkeptRenderSettings(sent: unknown, kept: unknown): Array<string> {
  return listUnkept(sent, kept, "");
}

/** The values a node takes of what was stored, or undefined for none. */
function parse(node: TSchemaNode, stored: unknown): unknown {
  if (isRenderSettingField(node)) {
    return toRenderSettingValue(node, stored);
  }

  const source: TValues = isRecord(stored) ? stored : {};

  return toGroup(node, (key: string, child: TSchemaNode) => parse(child, source[key]));
}

function resolve(node: TSchemaNode, base: unknown, overrides: unknown): unknown {
  if (isRenderSettingField(node)) {
    return overrides ?? base;
  }

  const [from, over] = [base as TValues, isRecord(overrides) ? overrides : {}];

  return toGroup(node, (key: string, child: TSchemaNode) => resolve(child, from[key], over[key])) ?? {};
}

function isSame(node: TSchemaNode, a: unknown, b: unknown): boolean {
  if (isRenderSettingField(node)) {
    return isSameRenderSetting(node, a, b);
  }

  return Object.entries(node).every(([key, child]: [string, TSchemaNode]) =>
    isSame(child, (a as TValues)[key], (b as TValues)[key])
  );
}

function merge(node: TSchemaNode, a: unknown, b: unknown): unknown {
  if (isRenderSettingField(node)) {
    return b ?? a;
  }

  const [first, second] = [isRecord(a) ? a : {}, isRecord(b) ? b : {}];

  return toGroup(node, (key: string, child: TSchemaNode) => merge(child, first[key], second[key]));
}

/** A group's values, each its key's; undefined where none is defined. */
function toGroup(
  node: { readonly [key: string]: TSchemaNode },
  toValue: (key: string, child: TSchemaNode) => unknown
): unknown {
  const group: TValues = {};

  for (const [key, child] of Object.entries(node)) {
    const value: unknown = toValue(key, child);

    if (value !== undefined) {
      group[key] = value;
    }
  }

  return Object.keys(group).length ? group : undefined;
}

function listUnkept(sent: unknown, kept: unknown, at: string): Array<string> {
  // Nothing given, which leaves the setting to what stands under it, is nothing lost.
  if (sent === undefined || sent === null) {
    return [];
  }

  if (isRecord(sent)) {
    const group: TValues = isRecord(kept) ? kept : {};

    return Object.entries(sent).flatMap(([key, value]: [string, unknown]) =>
      listUnkept(value, group[key], at ? `${at}.${key}` : key)
    );
  }

  return JSON.stringify(sent) === JSON.stringify(kept) ? [] : [at];
}
