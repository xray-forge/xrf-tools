import {
  IRenderChoiceField,
  IRenderFlagField,
  IRenderNumberField,
  IRenderWidthsField,
} from "@/core/render/lib/settings/render-setting-field";

/** What a setting of a type is described by: a field for a value, a field a key for a group of them. */
export type TRenderSettingSchema<T> = [T] extends [boolean]
  ? IRenderFlagField
  : [T] extends [number]
    ? IRenderNumberField
    : [T] extends [ReadonlyArray<number>]
      ? IRenderWidthsField
      : [T] extends [string]
        ? IRenderChoiceField<T>
        : { readonly [K in keyof T]-?: TRenderSettingSchema<T[K]> };
