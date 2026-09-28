import { IRendererFeatureSettings } from "@xrf/renderer";

import {
  ILevelFeatureOptions,
  ILevelFeatureState,
  TLevelFeatureKey,
} from "@/core/level/lib/features/level-feature-options";
import { BaseComponentProps } from "@/lib/dom/element-types";

/** What every feature group's toolbar popover takes. */
export interface ILevelFeatureActionProps<K extends TLevelFeatureKey> extends BaseComponentProps {
  isOn: boolean;
  /** The group as the view draws it while on, and whether the settings let it be on. */
  state: ILevelFeatureState<IRendererFeatureSettings[K]>;
  /** What the view sets over the settings, of which this group's part is changed. */
  features: ILevelFeatureOptions;
  onToggle: () => void;
  onChange: (features: ILevelFeatureOptions) => void;
}
