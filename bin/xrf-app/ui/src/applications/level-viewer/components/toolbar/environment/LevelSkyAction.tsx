import { default as NightsStayIcon } from "@mui/icons-material/NightsStay";
import { useInjection } from "@wirestate/react";
import { formatDegrees, formatPercent } from "@xrf/format";
import { LevelWeatherTexture } from "@xrf/ipc/types/xrf-app-levels";
import { ReactElement } from "react";

import { LevelWeatherTextureField } from "@/applications/level-viewer/components/weather/LevelWeatherTextureField";
import { LevelManualWeatherSlider } from "@/core/level/components/weather/LevelManualWeatherSlider";
import { LevelManualWeatherVectorField } from "@/core/level/components/weather/LevelManualWeatherVectorField";
import { LevelWeatherResetButton } from "@/core/level/components/weather/LevelWeatherResetButton";
import { ILevelManualWeather } from "@/core/level/lib/weather/level-manual-weather";
import { LEVEL_MANUAL_WEATHER_SKY_KEYS } from "@/core/level/lib/weather/level-manual-weather-groups";
import {
  describeLevelManualWeatherKey,
  LEVEL_MANUAL_WEATHER_LABELS,
} from "@/core/level/lib/weather/level-manual-weather-labels";
import { LevelViewService } from "@/core/level/services/level-view/level-view.service";
import { LevelWeatherService } from "@/core/level/services/level-weather/level-weather.service";
import { EditorPopoverGroup, EditorPopoverGroupSection } from "@/core/shell/editor/EditorPopoverGroup";
import { BaseComponentProps } from "@/lib/dom/element-types";

/** What the sky and clouds lists offer before the level's weather is read. */
const NO_TEXTURES: ReadonlyArray<LevelWeatherTexture> = [];

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
  const skies: ReadonlyArray<LevelWeatherTexture> = weatherService.description.value?.skies ?? NO_TEXTURES;
  const clouds: ReadonlyArray<LevelWeatherTexture> = weatherService.description.value?.clouds ?? NO_TEXTURES;
  const isCloudy: boolean = isClouded && manual.cloudsTexture !== "";

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
        onToggle={() => viewService.toggleOption("isSkyVisible")}
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
        onToggle={() => viewService.toggleOption("isClouded")}
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

      <LevelWeatherResetButton keys={LEVEL_MANUAL_WEATHER_SKY_KEYS} onReset={weatherService.resetManual} />
    </EditorPopoverGroup>
  );
}
