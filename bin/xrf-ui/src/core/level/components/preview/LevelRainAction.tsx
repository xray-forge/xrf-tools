import { default as WaterDropIcon } from "@mui/icons-material/WaterDrop";
import { ReactElement } from "react";

import { ThunderboltCollection } from "@/core/ipc/types/xrf-environment";
import { LevelManualWeatherSlider } from "@/core/level/components/weather/LevelManualWeatherSlider";
import { LevelManualWeatherVectorField } from "@/core/level/components/weather/LevelManualWeatherVectorField";
import { LevelWeatherCollectionSelect } from "@/core/level/components/weather/LevelWeatherCollectionSelect";
import { LevelWeatherResetButton } from "@/core/level/components/weather/LevelWeatherResetButton";
import { ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { ILevelManualWeather } from "@/core/level/lib/weather/level-manual-weather";
import { EditorPopoverGroup, EditorPopoverGroupSection } from "@/core/shell/editor/EditorPopoverGroup";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatSeconds } from "@/lib/format/duration";
import { formatPercent } from "@/lib/format/number";

/** The keys the popover sets. */
const RAIN_KEYS: ReadonlyArray<keyof ILevelManualWeather> = [
  "rainDensity",
  "rainColor",
  "thunderboltCollection",
  "thunderboltPeriod",
  "thunderboltDuration",
];

interface ILevelRainActionProps extends BaseComponentProps {
  options: ILevelViewOptions;
  /** The keyframe on screen: the one set by hand, or the weather's mix, which the first edit seeds it from. */
  manual: ILevelManualWeather;
  /** Every thunderbolt collection of the game. */
  collections: ReadonlyArray<ThunderboltCollection>;
  onToggle: (option: keyof ILevelViewOptions) => void;
  onEdit: (patch: Partial<ILevelManualWeather>) => void;
}

/**
 * How hard it rains and the streaks' colour, and the bolts that strike with it; whether either happens at all.
 */
export function LevelRainAction({
  "data-testid": dataTestId = "level-rain-action",
  id,
  className,
  options,
  manual,
  collections,
  onToggle,
  onEdit,
}: ILevelRainActionProps): ReactElement {
  const { isRainy, isThundering } = options;
  const isRaining: boolean = isRainy && manual.rainDensity > 0;
  const isStriking: boolean = isThundering && manual.thunderboltCollection !== "";

  return (
    <EditorPopoverGroup
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Rain"}
      description={[
        isRaining ? `Rain at ${formatPercent(manual.rainDensity)}` : isRainy ? "No rain" : "Rain off",
        isStriking ? `bolts of ${manual.thunderboltCollection}` : isThundering ? "no thunder" : "thunder off",
      ].join(", ")}
      icon={<WaterDropIcon />}
      isActive={isRaining || isStriking}
    >
      <EditorPopoverGroupSection label={"Rain"} isOn={isRainy} onToggle={() => onToggle("isRainy")}>
        <LevelManualWeatherSlider field={"rainDensity"} manual={manual} format={formatPercent} onEdit={onEdit} />

        <LevelManualWeatherVectorField field={"rainColor"} isColor manual={manual} onEdit={onEdit} />
      </EditorPopoverGroupSection>

      <EditorPopoverGroupSection label={"Thunder"} isOn={isThundering} onToggle={() => onToggle("isThundering")}>
        <LevelWeatherCollectionSelect
          value={manual.thunderboltCollection}
          collections={collections}
          onChange={(thunderboltCollection: string) => onEdit({ thunderboltCollection })}
        />

        <LevelManualWeatherSlider field={"thunderboltPeriod"} manual={manual} format={formatSeconds} onEdit={onEdit} />

        <LevelManualWeatherSlider
          field={"thunderboltDuration"}
          manual={manual}
          format={formatSeconds}
          onEdit={onEdit}
        />
      </EditorPopoverGroupSection>

      <LevelWeatherResetButton keys={RAIN_KEYS} onEdit={onEdit} />
    </EditorPopoverGroup>
  );
}
