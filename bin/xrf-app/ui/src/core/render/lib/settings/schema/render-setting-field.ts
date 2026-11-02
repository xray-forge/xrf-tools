import { RenderSettingField } from "@xrf/ipc/types/xrf-renderer";
import { clamp } from "@xrf/math";

import { TSettled } from "@/core/render/lib/settings/schema/render-settled";

/** What kind of value a setting takes, as the renderer's contract names it (`RenderSettingField.kind`). */
export enum ERenderSettingKind {
  FLAG = "flag",
  NUMBER = "number",
  CHOICE = "choice",
  /** A run of widths, as the sun's cascades are. */
  WIDTHS = "widths",
}

/** Any one setting's field, as the renderer's contract describes it beside the setting. */
export type TRenderSettingField = TSettled<RenderSettingField>;

/** A setting on or off. */
export type IRenderFlagField = Extract<TRenderSettingField, { kind: "flag" }>;

/** A number the engine takes between bounds, clamped to them, with the range xrf recommends and past which it wastes. */
export type IRenderNumberField = Extract<TRenderSettingField, { kind: "number" }>;

/** One of a set of values. */
export type IRenderChoiceField<T extends string> = Omit<Extract<TRenderSettingField, { kind: "choice" }>, "values"> & {
  readonly values: ReadonlyArray<T>;
};

/** A run of positive widths, each between bounds, as many as the most given. */
export type IRenderWidthsField = Extract<TRenderSettingField, { kind: "widths" }>;

/**
 * @param node - A schema's node.
 * @returns Whether it is one setting's field, rather than a group of them.
 */
export function isRenderSettingField(node: object): node is TRenderSettingField {
  return (
    "kind" in node &&
    Object.values(ERenderSettingKind).includes((node as TRenderSettingField).kind as ERenderSettingKind)
  );
}

/**
 * @param field - What a setting takes.
 * @param stored - What was stored for it.
 * @returns The value it takes from that, clamped where it is a number out of bounds; undefined for none it takes.
 */
export function toRenderSettingValue(field: TRenderSettingField, stored: unknown): unknown {
  switch (field.kind) {
    case ERenderSettingKind.FLAG:
      return typeof stored === "boolean" ? stored : undefined;

    case ERenderSettingKind.NUMBER:
      return typeof stored === "number" && Number.isFinite(stored) ? toBounded(field, stored) : undefined;

    case ERenderSettingKind.CHOICE:
      return field.values.find((value: string) => value === stored);

    case ERenderSettingKind.WIDTHS:
      return Array.isArray(stored) &&
        stored.length > 0 &&
        stored.length <= field.most &&
        stored.every((width: unknown) => typeof width === "number" && Number.isFinite(width) && width > 0)
        ? stored.map((width: number) => clamp(width, field.min, field.max))
        : undefined;
  }
}

/**
 * @param field - What a setting takes.
 * @param a - One value of it.
 * @param b - Another.
 * @returns Whether the two are the same setting: a run of widths width for width.
 */
export function isSameRenderSetting(field: TRenderSettingField, a: unknown, b: unknown): boolean {
  return field.kind === ERenderSettingKind.WIDTHS
    ? (a as ReadonlyArray<number>).join() === (b as ReadonlyArray<number>).join()
    : a === b;
}

function toBounded(field: IRenderNumberField, value: number): number {
  const bounded: number = clamp(value, field.min, field.max);

  return field.isInteger ? Math.round(bounded) : bounded;
}
