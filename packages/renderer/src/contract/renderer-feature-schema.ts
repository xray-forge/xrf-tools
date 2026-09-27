import {
  ERendererAmbientOcclusionQuality,
  ERendererAntialiasing,
  ERendererLightShadowFilter,
  ERendererRenderScale,
  IRendererFeatureSettings,
  RENDERER_MAX_SHADOW_CASCADES,
} from "#/contract/renderer-features";

/** What kind of value a setting takes. */
export enum ERendererSettingKind {
  FLAG = "flag",
  NUMBER = "number",
  CHOICE = "choice",
  /** A run of widths, as the sun's cascades are. */
  WIDTHS = "widths",
}

/** A setting on or off. */
export interface IRendererFlagField {
  readonly kind: ERendererSettingKind.FLAG;
}

/** A number the engine takes between bounds, clamped to them. */
export interface IRendererNumberField {
  readonly kind: ERendererSettingKind.NUMBER;
  readonly min: number;
  readonly max: number;
  /** Whether it is whole, as an engine's `CCC_Integer` is. */
  readonly isInteger?: boolean;
}

/** One of a set of values. */
export interface IRendererChoiceField<T extends string> {
  readonly kind: ERendererSettingKind.CHOICE;
  readonly values: ReadonlyArray<T>;
}

/** A run of positive widths, each between bounds, as many as the most given. */
export interface IRendererWidthsField {
  readonly kind: ERendererSettingKind.WIDTHS;
  readonly min: number;
  readonly max: number;
  readonly most: number;
}

/** Any one setting's field. */
export type TRendererSettingField =
  IRendererFlagField | IRendererNumberField | IRendererChoiceField<string> | IRendererWidthsField;

/** What a setting of a type is described by: a field for a value, a field a key for a group of them. */
export type TRendererSettingSchema<T> = [T] extends [boolean]
  ? IRendererFlagField
  : [T] extends [number]
    ? IRendererNumberField
    : [T] extends [ReadonlyArray<number>]
      ? IRendererWidthsField
      : [T] extends [string]
        ? IRendererChoiceField<T>
        : { readonly [K in keyof T]-?: TRendererSettingSchema<T[K]> };

const FLAG: IRendererFlagField = { kind: ERendererSettingKind.FLAG };

function toNumber(min: number, max: number, isInteger: boolean = false): IRendererNumberField {
  return { isInteger, kind: ERendererSettingKind.NUMBER, max, min };
}

function toChoice<T extends string>(values: Record<string, T>): IRendererChoiceField<T> {
  return { kind: ERendererSettingKind.CHOICE, values: Object.values(values) };
}

/**
 * Every feature setting, what it takes, and between which bounds: the engine console's own where the engine has the
 * setting (`xrRender_console.cpp`), the renderer's where it does not.
 */
export const RENDERER_FEATURE_SCHEMA: TRendererSettingSchema<IRendererFeatureSettings> = {
  ambientOcclusion: {
    isEnabled: FLAG,
    quality: toChoice(ERendererAmbientOcclusionQuality),
    radius: toNumber(0.1, 8),
    strength: toNumber(0, 2),
  },
  antialiasing: toChoice(ERendererAntialiasing),
  // The console's own bounds.
  exposure: {
    adaptation: toNumber(0.01, 10),
    amount: toNumber(0, 1),
    isEnabled: FLAG,
    lowLuminance: toNumber(0.0001, 1),
    middleGray: toNumber(0, 2),
  },
  grass: {
    // `r__detail_density`: a spacing, 0.1 the densest.
    density: toNumber(0.1, 0.99),
    height: toNumber(0.5, 2),
    isEnabled: FLAG,
    radius: toNumber(49, 300, true),
  },
  isGpuTimed: FLAG,
  isOcclusionCulled: FLAG,
  lights: {
    isEnabled: FLAG,
    isLevelLights: FLAG,
    isShadowed: FLAG,
    shadowFilter: toChoice(ERendererLightShadowFilter),
  },
  lod: {
    geometryLod: toNumber(0.1, 2),
    isImpostors: FLAG,
    ssaA: toNumber(16, 96),
    ssaB: toNumber(32, 64),
    ssaDiscard: toNumber(1, 10),
    ssaGlodEnd: toNumber(16, 96),
    ssaGlodStart: toNumber(128, 512),
  },
  shadows: {
    bias: toNumber(0, 5),
    blend: toNumber(0, 0.5),
    cascades: { kind: ERendererSettingKind.WIDTHS, max: 2000, min: 1, most: RENDERER_MAX_SHADOW_CASCADES },
    filter: toNumber(0, 3, true),
    isEnabled: FLAG,
    isStaggered: FLAG,
    reach: toNumber(0, 2000),
    resolution: toNumber(256, 8192, true),
  },
  upscaling: {
    scale: toChoice(ERendererRenderScale),
    sharpening: toNumber(0, 1),
  },
  water: {
    distortion: toNumber(0, 0.2),
    isDistorted: FLAG,
    isEnabled: FLAG,
    isSoft: FLAG,
    reflection: toNumber(0, 4),
    ripple: toNumber(0, 4),
    waveHeight: toNumber(0, 0.2),
    waveSpeed: toNumber(0, 100),
  },
};

/**
 * @param node - A schema's node.
 * @returns Whether it is one setting's field, rather than a group of them.
 */
export function isRendererSettingField(node: object): node is TRendererSettingField {
  return "kind" in node && Object.values(ERendererSettingKind).includes((node as TRendererSettingField).kind);
}

/**
 * @param field - What a setting takes.
 * @param stored - What was stored for it.
 * @returns The value it takes from that, clamped where it is a number out of bounds; undefined for none it takes.
 */
export function toRendererSettingValue(field: TRendererSettingField, stored: unknown): unknown {
  switch (field.kind) {
    case ERendererSettingKind.FLAG:
      return typeof stored === "boolean" ? stored : undefined;

    case ERendererSettingKind.NUMBER:
      return typeof stored === "number" && Number.isFinite(stored) ? toBounded(field, stored) : undefined;

    case ERendererSettingKind.CHOICE:
      return field.values.find((value: string) => value === stored);

    case ERendererSettingKind.WIDTHS:
      return Array.isArray(stored) &&
        stored.length > 0 &&
        stored.length <= field.most &&
        stored.every((width: unknown) => typeof width === "number" && Number.isFinite(width) && width > 0)
        ? stored.map((width: number) => Math.min(Math.max(width, field.min), field.max))
        : undefined;
  }
}

/**
 * @param field - What a setting takes.
 * @param a - One value of it.
 * @param b - Another.
 * @returns Whether the two are the same setting: a run of widths width for width.
 */
export function isSameRendererSetting(field: TRendererSettingField, a: unknown, b: unknown): boolean {
  return field.kind === ERendererSettingKind.WIDTHS
    ? (a as ReadonlyArray<number>).join() === (b as ReadonlyArray<number>).join()
    : a === b;
}

function toBounded(field: IRendererNumberField, value: number): number {
  const bounded: number = Math.min(Math.max(value, field.min), field.max);

  return field.isInteger ? Math.round(bounded) : bounded;
}
