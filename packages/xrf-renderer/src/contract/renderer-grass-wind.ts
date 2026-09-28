import { IRendererGrassSwing } from "#/contract/renderer-grass-swing";

/**
 * How the grass sways (`CDetailManager::swing_desc`): between its normal and its fast swing, by the weather's wind
 * strength.
 */
export interface IRendererGrassWind {
  /** `wind_strength_factor`, from the normal swing at zero to the fast one at one. */
  strength: number;
  normal: IRendererGrassSwing;
  fast: IRendererGrassSwing;
}
