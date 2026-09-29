import { ILevelManualWeather } from "@/core/level/lib/weather/level-manual-weather";
import { BaseComponentProps } from "@/lib/dom/element-types";

/** What a toolbar popover editing the weather's keys is given. */
export interface ILevelManualWeatherActionProps extends BaseComponentProps {
  /** Whether what it draws is drawn, which is what the toggle turns over. */
  isOn: boolean;
  /** The keyframe on screen: the one set by hand, or the weather's mix, which the first edit seeds it from. */
  manual: ILevelManualWeather;
  onToggle: () => void;
  onEdit: (patch: Partial<ILevelManualWeather>) => void;
}
