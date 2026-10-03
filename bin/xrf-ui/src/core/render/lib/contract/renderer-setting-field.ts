import { clamp } from "@xrf/math";

import { IRendererChoiceField } from "@/core/render/lib/contract/renderer-choice-field";
import { IRendererFlagField } from "@/core/render/lib/contract/renderer-flag-field";
import { IRendererNumberField } from "@/core/render/lib/contract/renderer-number-field";
import { IRendererWidthsField } from "@/core/render/lib/contract/renderer-widths-field";

/** What kind of value a setting takes. */
export enum ERendererSettingKind {
  FLAG = "flag",
  NUMBER = "number",
  CHOICE = "choice",
  /** A run of widths, as the sun's cascades are. */
  WIDTHS = "widths",
}

/** Any one setting's field. */
export type TRendererSettingField =
  IRendererFlagField | IRendererNumberField | IRendererChoiceField<string> | IRendererWidthsField;

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
export function isSameRendererSetting(field: TRendererSettingField, a: unknown, b: unknown): boolean {
  return field.kind === ERendererSettingKind.WIDTHS
    ? (a as ReadonlyArray<number>).join() === (b as ReadonlyArray<number>).join()
    : a === b;
}

function toBounded(field: IRendererNumberField, value: number): number {
  const bounded: number = clamp(value, field.min, field.max);

  return field.isInteger ? Math.round(bounded) : bounded;
}
