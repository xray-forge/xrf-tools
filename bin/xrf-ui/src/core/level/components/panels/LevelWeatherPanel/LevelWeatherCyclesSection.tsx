import { Nullable } from "@xrf/types";
import { ReactElement } from "react";

import { LevelWeatherCycleSelect } from "@/core/level/components/weather/LevelWeatherCycleSelect";
import { ILevelWeatherCycleChoice } from "@/core/level/lib/weather/level-weather-cycle-choice";
import { EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ILevelWeatherCyclesSectionProps extends BaseComponentProps {
  cycles: ReadonlyArray<ILevelWeatherCycleChoice>;
  /** The cycle chosen, or null for none. */
  selected: Nullable<string>;
  /** The cycle being read, or null while none is. */
  reading: Nullable<string>;
  onSelect: (name: string) => void;
}

/**
 * Every cycle the level can be played under: the ones its weather resolves to first, then the rest of the game's.
 */
export function LevelWeatherCyclesSection({
  "data-testid": dataTestId = "level-weather-cycles-section",
  id,
  className,
  cycles,
  selected,
  reading,
  onSelect,
}: ILevelWeatherCyclesSectionProps): ReactElement {
  return (
    <EditorPanelSection data-testid={dataTestId} id={id} className={className} title={"Cycle"}>
      <LevelWeatherCycleSelect cycles={cycles} selected={selected} reading={reading} onSelect={onSelect} />
    </EditorPanelSection>
  );
}
