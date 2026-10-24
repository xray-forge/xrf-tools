import { default as WbSunnyIcon } from "@mui/icons-material/WbSunny";
import { Button, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { formatCount, formatDegrees } from "@xrf/format";
import { Nullable } from "@xrf/types";
import { ReactElement, useCallback, useMemo } from "react";

import { LevelSunDescription } from "@/core/ipc/types/xrf-app-levels";
import { ERenderVolumetricMode } from "@/core/ipc/types/xrf-renderer";
import { useLevelRenderFeatures } from "@/core/level/components/preview/toolbar/use-level-render-features";
import { LevelManualWeatherSlider } from "@/core/level/components/weather/LevelManualWeatherSlider";
import { LevelManualWeatherVectorField } from "@/core/level/components/weather/LevelManualWeatherVectorField";
import { LevelWeatherResetButton } from "@/core/level/components/weather/LevelWeatherResetButton";
import { LevelWeatherSunSelect } from "@/core/level/components/weather/LevelWeatherSunSelect";
import { toLevelModeFeature } from "@/core/level/lib/features/level-feature-options";
import { ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { ILevelManualWeather, toLevelManualSun } from "@/core/level/lib/weather/level-manual-weather";
import { LEVEL_MANUAL_WEATHER_SUN_KEYS } from "@/core/level/lib/weather/level-manual-weather-groups";
import {
  ILevelSunShaftsOptions,
  LEVEL_SUN_SHAFTS_MINIMUM_LIMITS,
  LEVEL_SUN_SHAFTS_STEPS_LIMITS,
} from "@/core/level/lib/weather/level-sun-shafts-options";
import { LevelLoadService } from "@/core/level/services/level-load.service";
import { LevelRenderService } from "@/core/level/services/level-render.service";
import { LevelViewService } from "@/core/level/services/level-view.service";
import { LevelWeatherService } from "@/core/level/services/level-weather.service";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import { EditorPopoverAction } from "@/core/shell/editor/EditorPopoverAction";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { BaseComponentProps } from "@/lib/dom/element-types";

/** Lens flares while the weather is not read yet. */
const NO_SUNS: ReadonlyArray<string> = [];

/**
 * The sun, the ambient and the hemisphere, by the keys a weather writes them with; the lens flare the sun is drawn
 * with and the light shafts through its shadow, each with its switch. Its direction is marked among the overlays.
 */
export function LevelSunAction({
  "data-testid": dataTestId = "level-sun-action",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const viewService: LevelViewService = useInjection(LevelViewService);
  const weatherService: LevelWeatherService = useInjection(LevelWeatherService);
  const loadService: LevelLoadService = useInjection(LevelLoadService);
  const renderService: LevelRenderService = useInjection(LevelRenderService);
  const manual: ILevelManualWeather = weatherService.shown;
  const { settings, features } = useLevelRenderFeatures();
  // The enhanced volumetrics draw the sun's shafts in the engine's march's place, which its steps then leave alone.
  const isMarched: boolean =
    toLevelModeFeature("volumetric", settings, features).mode !== ERenderVolumetricMode.ENHANCED;
  // The sun the open level was compiled against, or null where it names none.
  const sun: Nullable<LevelSunDescription> = loadService.level.value?.selected.value.sun ?? null;
  // The sun or moon the sky draws now, by its `suns.ltx` section, or null where it shows neither.
  const drawnSun: Nullable<string> = renderService.drawnSun;
  // Every lens flare of the game a keyframe can name by `sun`.
  const suns: ReadonlyArray<string> = weatherService.description.value?.suns ?? NO_SUNS;
  const options: ILevelViewOptions = viewService.options;
  // How the level's sun shafts step, and the floor under their density.
  const sunShafts: ILevelSunShaftsOptions = weatherService.sunShafts;
  const compiled = useMemo(() => toLevelManualSun(sun?.direction ?? null), [sun]);

  // `setHP` turns the light's heading by the altitude and tilts it down by the longitude.
  const reading: string = `${formatDegrees(-manual.sunLongitude)} up at a bearing of ${formatDegrees(manual.sunAltitude)}`;

  // The occlusion a level carries was computed for its sun, so lighting it from there is lighting it as xrLC assumed.
  const onUseCompiled = useCallback(() => {
    if (compiled) {
      weatherService.editManual(compiled);
    }
  }, [compiled, weatherService]);

  return (
    <EditorPopoverAction
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Sun"}
      description={drawnSun ? `Sun ${reading}, ${drawnSun} in the sky` : `Sun ${reading}, none in the sky`}
      icon={<WbSunnyIcon />}
      isActive={drawnSun !== null}
    >
      <div className={"flex w-60 flex-col gap-2 px-4 py-2"}>
        <Typography className={"text-text-secondary"} variant={"overline"}>
          Sun
        </Typography>

        <LevelManualWeatherVectorField field={"sunColor"} isColor manual={manual} onEdit={weatherService.editManual} />

        <LevelManualWeatherSlider
          field={"sunAltitude"}
          manual={manual}
          format={formatDegrees}
          onEdit={weatherService.editManual}
        />

        <LevelManualWeatherSlider
          field={"sunLongitude"}
          manual={manual}
          format={formatDegrees}
          onEdit={weatherService.editManual}
        />

        <Typography className={"block text-text-secondary"} variant={"caption"}>
          {`The sun stands ${reading}.`}
        </Typography>

        <LevelManualWeatherVectorField
          field={"ambientColor"}
          isColor
          manual={manual}
          onEdit={weatherService.editManual}
        />

        <LevelManualWeatherVectorField
          field={"hemisphereColor"}
          isColor
          manual={manual}
          onEdit={weatherService.editManual}
        />

        <Button size={"small"} disabled={!compiled} onClick={onUseCompiled}>
          {compiled ? "Use the level's compiled sun" : "This level names no sun"}
        </Button>

        <Typography className={"text-text-secondary"} variant={"overline"}>
          Lens flare
        </Typography>

        <CheckboxFormRow
          label={"Lens flares"}
          description={"Draws the flares over the frame; the sun's sprite and glow draw either way"}
          isChecked={options.isLensFlared}
          onChange={() => viewService.toggleOption("isLensFlared")}
        />

        <LevelWeatherSunSelect
          value={manual.sun}
          suns={suns}
          onChange={(sun: string) => weatherService.editManual({ sun })}
        />

        <Typography className={"text-text-secondary"} variant={"overline"}>
          Sunshafts
        </Typography>

        <CheckboxFormRow
          label={"Sun shafts"}
          description={"Draws the sun's light through the air, cut by its shadow"}
          isChecked={options.isSunShafted}
          onChange={() => viewService.toggleOption("isSunShafted")}
        />

        <LevelManualWeatherSlider
          field={"sunShaftsIntensity"}
          manual={manual}
          format={(value: number) => value.toFixed(2)}
          onEdit={weatherService.editManual}
        />

        <RenderValueSlider
          label={"Steps"}
          hint={"Game setting: r2_sunshafts_quality steps 20, 20 and 40 (OpenXRay), 15, 25 and 30 (Monolith)"}
          value={sunShafts.steps}
          isDisabled={!isMarched}
          {...LEVEL_SUN_SHAFTS_STEPS_LIMITS}
          format={formatCount}
          onChange={(steps: number) => weatherService.setSunShafts({ ...sunShafts, steps })}
        />

        {isMarched ? null : (
          <p className={"text-xs text-text-secondary"}>
            The enhanced volumetrics draw the shafts; their steps apply with Engine volumetrics.
          </p>
        )}

        <RenderValueSlider
          label={"Minimum"}
          hint={"Game setting: r2_sunshafts_min"}
          value={sunShafts.minimum}
          {...LEVEL_SUN_SHAFTS_MINIMUM_LIMITS}
          format={(value: number) => value.toFixed(2)}
          onChange={(minimum: number) => weatherService.setSunShafts({ ...sunShafts, minimum })}
        />

        <LevelWeatherResetButton keys={LEVEL_MANUAL_WEATHER_SUN_KEYS} onReset={weatherService.resetManual} />
      </div>
    </EditorPopoverAction>
  );
}
