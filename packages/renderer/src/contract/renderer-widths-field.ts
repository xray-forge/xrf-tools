import { ERendererSettingKind } from "#/contract/renderer-setting-field";

/** A run of positive widths, each between bounds, as many as the most given. */
export interface IRendererWidthsField {
  readonly kind: ERendererSettingKind.WIDTHS;
  readonly min: number;
  readonly max: number;
  readonly most: number;
}
