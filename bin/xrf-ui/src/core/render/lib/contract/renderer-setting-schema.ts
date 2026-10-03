import { IRendererChoiceField } from "@/core/render/lib/contract/renderer-choice-field";
import { IRendererFlagField } from "@/core/render/lib/contract/renderer-flag-field";
import { IRendererNumberField } from "@/core/render/lib/contract/renderer-number-field";
import { IRendererWidthsField } from "@/core/render/lib/contract/renderer-widths-field";

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
