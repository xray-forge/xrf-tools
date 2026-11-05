import { useInjection } from "@wirestate/react";
import { EMPTY_ARRAY } from "@xrf/collections";
import { formatDegrees, formatPercent } from "@xrf/format";
import { LevelWeatherTexture } from "@xrf/ipc/types/xrf-app-levels";
import { Optional } from "@xrf/types";
import { EditorPopoverGroup, EditorPopoverGroupSection } from "@xrf/ui/editor/popover-group";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { NightsStayIcon } from "@xrf/ui/icons/nights-stay";
import { ReactElement } from "react";

import { LevelManualWeatherSlider } from "@/core/level/components/weather/LevelManualWeatherSlider";
import { LevelManualWeatherVectorField } from "@/core/level/components/weather/LevelManualWeatherVectorField";
import {
  LEVEL_WEATHER_RESET_LABEL,
  useLevelWeatherReset,
} from "@/core/level/components/weather/use-level-weather-reset";
import { ILevelManualWeather } from "@/core/level/lib/weather/level-manual-weather";
import {
  LEVEL_MANUAL_WEATHER_CLOUDS_KEYS,
  LEVEL_MANUAL_WEATHER_SKY_KEYS,
} from "@/core/level/lib/weather/level-manual-weather-groups";
import {
  describeLevelManualWeatherKey,
  LEVEL_MANUAL_WEATHER_LABELS,
} from "@/core/level/lib/weather/level-manual-weather-labels";
import { LevelViewService } from "@/core/level/services/level-view";
import { LevelWeatherService } from "@/core/level/services/level-weather";
import { LevelWeatherTextureField } from "@/tools/level-viewer/components/weather/LevelWeatherTextureField";

/**
 * The sky cube, its tint and turn, which its `#small` twin follows, and the clouds over it with their cover.
 */
export function LevelSkyAction({
  "data-testid": dataTestId = "level-sky-action",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const viewService: LevelViewService = useInjection(LevelViewService);
  const weatherService: LevelWeatherService = useInjection(LevelWeatherService);

  const { isSkyVisible, isClouded } = viewService.options;
  // The keyframe on screen: the one set by hand, or the weather's mix, which the first edit seeds it from.
  const manual: ILevelManualWeather = weatherService.shown;
  const skies: ReadonlyArray<LevelWeatherTexture> = weatherService.description.value?.skies ?? EMPTY_ARRAY;
  const clouds: ReadonlyArray<LevelWeatherTexture> = weatherService.description.value?.clouds ?? EMPTY_ARRAY;
  const isCloudy: boolean = isClouded && manual.cloudsTexture !== "";
  const onSkyReset: Optional<() => void> = useLevelWeatherReset(LEVEL_MANUAL_WEATHER_SKY_KEYS);
  const onCloudsReset: Optional<() => void> = useLevelWeatherReset(LEVEL_MANUAL_WEATHER_CLOUDS_KEYS);

  return (
    <EditorPopoverGroup
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Sky"}
      description={[
        isSkyVisible ? `Sky ${manual.skyTexture || "none"}` : "Sky off, the backdrop behind the level",
        isCloudy ? `clouds ${manual.cloudsTexture}, ${formatPercent(manual.cloudsColor[3])} cover` : "no clouds",
      ].join(", ")}
      icon={<NightsStayIcon />}
      isActive={isSkyVisible || isCloudy}
    >
      <EditorPopoverGroupSection
        label={"Sky"}
        isOn={isSkyVisible}
        resetLabel={LEVEL_WEATHER_RESET_LABEL}
        onToggle={() => viewService.toggleOption("isSkyVisible")}
        onReset={onSkyReset}
      >
        <LevelWeatherTextureField
          label={LEVEL_MANUAL_WEATHER_LABELS.skyTexture}
          hint={describeLevelManualWeatherKey("skyTexture")}
          value={manual.skyTexture}
          textures={skies}
          onChange={(skyTexture: string) => weatherService.editManual({ skyTexture })}
        />

        <LevelManualWeatherVectorField field={"skyColor"} isColor manual={manual} onEdit={weatherService.editManual} />

        <LevelManualWeatherSlider
          field={"skyRotation"}
          manual={manual}
          format={formatDegrees}
          onEdit={weatherService.editManual}
        />
      </EditorPopoverGroupSection>

      <EditorPopoverGroupSection
        label={"Clouds"}
        isOn={isClouded}
        resetLabel={LEVEL_WEATHER_RESET_LABEL}
        onToggle={() => viewService.toggleOption("isClouded")}
        onReset={onCloudsReset}
      >
        <LevelWeatherTextureField
          label={LEVEL_MANUAL_WEATHER_LABELS.cloudsTexture}
          hint={describeLevelManualWeatherKey("cloudsTexture")}
          value={manual.cloudsTexture}
          textures={clouds}
          onChange={(cloudsTexture: string) => weatherService.editManual({ cloudsTexture })}
        />

        <LevelManualWeatherVectorField
          field={"cloudsColor"}
          isColor
          manual={manual}
          onEdit={weatherService.editManual}
        />

        <LevelManualWeatherSlider
          field={"cloudsRotation"}
          manual={manual}
          format={formatDegrees}
          onEdit={weatherService.editManual}
        />
      </EditorPopoverGroupSection>
    </EditorPopoverGroup>
  );
}
