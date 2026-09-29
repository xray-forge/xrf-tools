import { Nullable } from "@xrf/types";

import { IRendererThunderboltGradient } from "#/contract/weather/renderer-thunderbolt-gradient";

/**
 * A bolt of a collection, as `SThunderboltDesc` loads it.
 */
export interface IRendererThunderbolt {
  /** Its model, by index among the thunder's models; null for one that did not read, which draws nothing. */
  model: Nullable<number>;
  /** Its `color_anim`, by index among the thunder's animators; null for one the library lacks, which flashes black. */
  color: Nullable<number>;
  top: IRendererThunderboltGradient;
  center: IRendererThunderboltGradient;
}
