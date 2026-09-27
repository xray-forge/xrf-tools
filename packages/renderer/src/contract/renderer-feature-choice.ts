import {
  isRendererSettingField,
  isSameRendererSetting,
  RENDERER_FEATURE_SCHEMA,
  toRendererSettingValue,
  TRendererSettingField,
} from "#/contract/renderer-feature-schema";
import { ERendererPreset, IRendererFeatureSettings, RENDERER_PRESETS } from "#/contract/renderer-features";

/** A group's override: part of the group, or the whole of a setting that is one value. */
type TRendererFeatureOverride<T> = T extends ReadonlyArray<unknown> ? T : T extends object ? Partial<T> : T;

/** Every feature's override, from the settings themselves, so the two cannot drift. */
type TRendererFeatureOverrideGroups = {
  [K in keyof IRendererFeatureSettings]?: TRendererFeatureOverride<IRendererFeatureSettings[K]>;
};

/** What was changed on top of a preset, feature by feature. */
export interface IRendererFeatureOverrides extends TRendererFeatureOverrideGroups {}

/** A preset and what was changed on top of it, which is what a consumer stores. */
export interface IRendererFeatureChoice {
  preset: ERendererPreset;
  overrides: IRendererFeatureOverrides;
}

export const DEFAULT_RENDERER_FEATURE_CHOICE: IRendererFeatureChoice = {
  overrides: {},
  preset: ERendererPreset.BASE,
};

/** A node of the schema: one setting's field, or a group of nodes by key. */
type TSchemaNode = TRendererSettingField | { readonly [key: string]: TSchemaNode };

/** A group of values by key, as a group of settings or of their overrides is. */
type TValues = Record<string, unknown>;

const SCHEMA: TSchemaNode = RENDERER_FEATURE_SCHEMA as unknown as TSchemaNode;

/**
 * @param stored - What a consumer stored for its choice, parsed from wherever it keeps it.
 * @returns The choice, every value the features do not take dropped and every number held to its bounds; the default
 *   for nothing usable.
 */
export function toRendererFeatureChoice(stored: unknown): IRendererFeatureChoice {
  if (!isValues(stored)) {
    return DEFAULT_RENDERER_FEATURE_CHOICE;
  }

  return {
    overrides: (parse(SCHEMA, stored.overrides) ?? {}) as IRendererFeatureOverrides,
    preset: Object.values(ERendererPreset).find((it) => it === stored.preset) ?? DEFAULT_RENDERER_FEATURE_CHOICE.preset,
  };
}

/**
 * @param choice - A preset and what was changed on top of it.
 * @returns Every feature as the choice sets it.
 */
export function resolveRendererFeatures(choice: IRendererFeatureChoice): IRendererFeatureSettings {
  return resolve(SCHEMA, RENDERER_PRESETS[choice.preset], choice.overrides) as IRendererFeatureSettings;
}

/**
 * @param choice - A preset and what was changed on top of it.
 * @returns Whether the features differ from the preset's, which a settings view shows as custom.
 */
export function isRendererFeatureChoiceCustom(choice: IRendererFeatureChoice): boolean {
  return !isSame(SCHEMA, resolveRendererFeatures(choice), RENDERER_PRESETS[choice.preset]);
}

/**
 * @param overrides - What was changed on top of a preset.
 * @param changes - What changes now, setting by setting.
 * @returns The two together, each change in place of what it changes.
 */
export function mergeRendererFeatureOverrides(
  overrides: IRendererFeatureOverrides,
  changes: IRendererFeatureOverrides
): IRendererFeatureOverrides {
  return (merge(SCHEMA, overrides, changes) ?? {}) as IRendererFeatureOverrides;
}

/** The values a node takes of what was stored, or undefined for none. */
function parse(node: TSchemaNode, stored: unknown): unknown {
  if (isRendererSettingField(node)) {
    return toRendererSettingValue(node, stored);
  }

  const source: TValues = isValues(stored) ? stored : {};

  return toGroup(node, (key: string, child: TSchemaNode) => parse(child, source[key]));
}

function resolve(node: TSchemaNode, base: unknown, overrides: unknown): unknown {
  if (isRendererSettingField(node)) {
    return overrides ?? base;
  }

  const [from, over] = [base as TValues, isValues(overrides) ? overrides : {}];

  return toGroup(node, (key: string, child: TSchemaNode) => resolve(child, from[key], over[key])) ?? {};
}

function isSame(node: TSchemaNode, a: unknown, b: unknown): boolean {
  if (isRendererSettingField(node)) {
    return isSameRendererSetting(node, a, b);
  }

  return Object.entries(node).every(([key, child]) => isSame(child, (a as TValues)[key], (b as TValues)[key]));
}

function merge(node: TSchemaNode, a: unknown, b: unknown): unknown {
  if (isRendererSettingField(node)) {
    return b ?? a;
  }

  const [first, second] = [isValues(a) ? a : {}, isValues(b) ? b : {}];

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

function isValues(value: unknown): value is TValues {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
