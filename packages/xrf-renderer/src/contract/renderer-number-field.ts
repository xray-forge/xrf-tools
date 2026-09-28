import { ERendererSettingKind } from "#/contract/renderer-setting-field";

/** A number the engine takes between bounds, clamped to them. */
export interface IRendererNumberField {
  readonly kind: ERendererSettingKind.NUMBER;
  readonly min: number;
  readonly max: number;
  /** Whether it is whole, as an engine's `CCC_Integer` is. */
  readonly isInteger?: boolean;
}
