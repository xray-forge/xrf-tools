import { ERendererSettingKind } from "#/contract/renderer-setting-field";

/** One of a set of values. */
export interface IRendererChoiceField<T extends string> {
  readonly kind: ERendererSettingKind.CHOICE;
  readonly values: ReadonlyArray<T>;
}
