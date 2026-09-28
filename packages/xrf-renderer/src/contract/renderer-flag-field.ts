import { ERendererSettingKind } from "#/contract/renderer-setting-field";

/** A setting on or off. */
export interface IRendererFlagField {
  readonly kind: ERendererSettingKind.FLAG;
}
