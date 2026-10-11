import { default as NightsStayIcon } from "@mui/icons-material/NightsStay";
import { Button } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { formatDegrees, formatPercent } from "@xrf/format";
import { ReactElement } from "react";

import { LevelWeatherTexture } from "@/core/ipc/types/xrf-app";
import { ERenderDebandingMode, RenderDebandingQuality } from "@/core/ipc/types/xrf-renderer";
import { useLevelFeatureOverride } from "@/core/level/components/preview/toolbar/use-level-feature-override";
import { useLevelRenderFeatures } from "@/core/level/components/preview/toolbar/use-level-render-features";
import { LevelManualWeatherSlider } from "@/core/level/components/weather/LevelManualWeatherSlider";
import { LevelManualWeatherVectorField } from "@/core/level/components/weather/LevelManualWeatherVectorField";
import { LevelWeatherResetButton } from "@/core/level/components/weather/LevelWeatherResetButton";
import { LevelWeatherTextureField } from "@/core/level/components/weather/LevelWeatherTextureField";
import { toLevelModeFeature } from "@/core/level/lib/features/level-feature-options";
import { ILevelManualWeather } from "@/core/level/lib/weather/level-manual-weather";
import { LEVEL_MANUAL_WEATHER_SKY_KEYS } from "@/core/level/lib/weather/level-manual-weather-groups";
import { LevelViewService } from "@/core/level/services/level-view.service";
import { LevelWeatherService } from "@/core/level/services/level-weather.service";
import { RenderValueChoice } from "@/core/render/components/controls/RenderValueChoice";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import {
  describeRenderDebandingQuality,
  explainRenderDebandingMode,
  formatDebandingRadius,
  RENDER_DEBANDING_LIMITS,
  RENDER_DEBANDING_QUALITY_OPTIONS,
} from "@/core/render/lib/features/render-debanding-choices";
import { TRenderDebandingSettings } from "@/core/render/lib/settings/render-feature-settings";
import { EditorPopoverGroup, EditorPopoverGroupSection } from "@/core/shell/editor/EditorPopoverGroup";
import { BaseComponentProps } from "@/lib/dom/element-types";

/** What the sky and clouds lists offer before the level's weather is read. */
const NO_TEXTURES: ReadonlyArray<LevelWeatherTexture> = [];

/**
 * The sky cube, its tint and turn, which its `#small` twin follows, the clouds over it with their cover, and whether
 * its colour bands are smoothed.
 */
export function LevelSkyAction({
  "data-testid": dataTestId = "level-sky-action",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const viewService: LevelViewService = useInjection(LevelViewService);
  const weatherService: LevelWeatherService = useInjection(LevelWeatherService);
  const { settings, features } = useLevelRenderFeatures();
  const { set: setDebanding, reset: resetDebanding } = useLevelFeatureOverride("debanding");

  const { isSkyVisible, isClouded } = viewService.options;
  // The keyframe on screen: the one set by hand, or the weather's mix, which the first edit seeds it from.
  const manual: ILevelManualWeather = weatherService.shown;
  const skies: ReadonlyArray<LevelWeatherTexture> = weatherService.description.value?.skies ?? NO_TEXTURES;
  const clouds: ReadonlyArray<LevelWeatherTexture> = weatherService.description.value?.clouds ?? NO_TEXTURES;
  // Whether and how the view smooths the sky's colour bands.
  const debanding: TRenderDebandingSettings = toLevelModeFeature("debanding", settings, features);
  const isCloudy: boolean = isClouded && manual.cloudsTexture !== "";
  const isDebanded: boolean = debanding.mode === ERenderDebandingMode.ENHANCED;

  return (
    <EditorPopoverGroup
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Sky"}
      description={[
        isSkyVisible ? `Sky ${manual.skyTexture || "none"}` : "Sky off, the backdrop behind the level",
        isCloudy ? `clouds ${manual.cloudsTexture}, ${formatPercent(manual.cloudsColor[3])} cover` : "no clouds",
        ...(isDebanded
          ? [`debanded at ${describeRenderDebandingQuality(debanding.quality).toLowerCase()} quality`]
          : []),
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
          label={"sky_texture"}
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
          label={"clouds_texture"}
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

      <LevelWeatherResetButton keys={LEVEL_MANUAL_WEATHER_SKY_KEYS} onEdit={weatherService.editManual} />

      <EditorPopoverGroupSection
        label={"Debanding"}
        description={explainRenderDebandingMode(debanding.mode)}
        isOn={isDebanded}
        onToggle={() =>
          setDebanding({ mode: isDebanded ? ERenderDebandingMode.ENGINE : ERenderDebandingMode.ENHANCED })
        }
      >
        {isDebanded ? (
          <>
            <RenderValueChoice
              label={"Quality"}
              options={RENDER_DEBANDING_QUALITY_OPTIONS}
              value={debanding.quality}
              onChange={(quality: RenderDebandingQuality) => setDebanding({ quality })}
            />

            <RenderValueSlider
              label={"Radius"}
              value={debanding.radius}
              {...RENDER_DEBANDING_LIMITS.radius}
              format={formatDebandingRadius}
              onChange={(radius: number) => setDebanding({ radius })}
            />
          </>
        ) : null}

        <Button size={"small"} onClick={resetDebanding}>
          Back to the settings for the debanding
        </Button>
      </EditorPopoverGroupSection>
    </EditorPopoverGroup>
  );
}
