import { Nullable } from "@xrf/types";

import { IRendererLightAnimator } from "#/contract/scene/renderer-light-animator";
import { IRendererThunderSettings } from "#/contract/weather/renderer-thunder-settings";
import { IRendererThunderbolt } from "#/contract/weather/renderer-thunderbolt";
import { IRendererThunderboltModel } from "#/contract/weather/renderer-thunderbolt-model";

/**
 * What a weather strikes with (`CEffect_Thunderbolt`): its collections, their bolts, and where bolts strike.
 */
export interface IRendererThunder {
  /** Where bolts strike, or null for a game that says nowhere, which strikes nothing. */
  settings: Nullable<IRendererThunderSettings>;
  /** Every collection by name, its bolts by name in the order written. */
  collections: Readonly<Record<string, ReadonlyArray<string>>>;
  /** Every bolt a collection names that the game has, by name. */
  bolts: Readonly<Record<string, IRendererThunderbolt>>;
  models: ReadonlyArray<IRendererThunderboltModel>;
  animators: ReadonlyArray<IRendererLightAnimator>;
}
