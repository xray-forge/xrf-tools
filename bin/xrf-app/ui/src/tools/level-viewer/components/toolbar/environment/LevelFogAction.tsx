import { useInjection } from "@wirestate/react";
import { formatNumber, formatPercent } from "@xrf/format";
import { ERenderFogMode } from "@xrf/ipc/types/xrf-renderer";
import { Optional } from "@xrf/types";
import { EditorPopoverGroupSection } from "@xrf/ui/editor/popover-group";
import { EditorPopoverToggle } from "@xrf/ui/editor/popover-toggle";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { IChoiceFormRowOption } from "@xrf/ui/form/choice-row";
import { FoggyIcon } from "@xrf/ui/icons/foggy";
import { ReactElement } from "react";

import { useLevelFeatureOverride } from "@/core/level/components/preview/toolbar/use-level-feature-override";
import {
  LEVEL_SETTINGS_RESET_LABEL,
  useLevelFeatureReset,
} from "@/core/level/components/preview/toolbar/use-level-feature-reset";
import { useLevelRenderFeatures } from "@/core/level/components/preview/toolbar/use-level-render-features";
import { useLevelViewToggle } from "@/core/level/components/preview/toolbar/use-level-view-toggle";
import { LevelManualWeatherSlider } from "@/core/level/components/weather/LevelManualWeatherSlider";
import { LevelManualWeatherVectorField } from "@/core/level/components/weather/LevelManualWeatherVectorField";
import {
  LEVEL_WEATHER_RESET_LABEL,
  useLevelWeatherReset,
} from "@/core/level/components/weather/use-level-weather-reset";
import { toLevelModeFeature } from "@/core/level/lib/features/level-feature-options";
import { ILevelManualWeather } from "@/core/level/lib/weather/level-manual-weather";
import { LEVEL_MANUAL_WEATHER_FOG_KEYS } from "@/core/level/lib/weather/level-manual-weather-groups";
import { LevelViewService } from "@/core/level/services/level-view";
import { LevelWeatherService } from "@/core/level/services/level-weather";
import { RenderValueChoice } from "@/core/render/components/controls/RenderValueChoice";
import { RenderSettingEntries } from "@/core/render/components/settings/RenderSettingEntries";
import { RENDER_FOG_SETTINGS } from "@/core/render/lib/features/world/render-fog-choices";
import { TRenderFogSettings } from "@/core/render/lib/settings/choice/render-feature-settings";

/** What the distance fades into: the sky as the engine fades it, or its haze. */
const FADE_OPTIONS: ReadonlyArray<IChoiceFormRowOption<"sky" | "haze">> = [
  { label: "Sky (engine)", value: "sky" },
  { label: "Sky haze", value: "haze" },
];

/**
 * The fog closing the level in, where the view ends, and what the distance fades into; whether it is drawn, and
 * whether as the engine draws it or as the enhanced fog, thickened low down and scattering the frame's light.
 */
export function LevelFogAction({
  "data-testid": dataTestId = "level-fog-action",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const viewService: LevelViewService = useInjection(LevelViewService);
  const weatherService: LevelWeatherService = useInjection(LevelWeatherService);
  const { settings, features, drawn } = useLevelRenderFeatures();
  const { set } = useLevelFeatureOverride("fog");
  const onSettingsReset: Optional<() => void> = useLevelFeatureReset("fog");
  const onWeatherReset: Optional<() => void> = useLevelWeatherReset(LEVEL_MANUAL_WEATHER_FOG_KEYS);
  const { isOn, toggle } = useLevelViewToggle("isFogged");

  const manual: ILevelManualWeather = weatherService.shown;
  // Whether the distance fades into the sky's haze rather than into the sky itself.
  const isHazed: boolean = viewService.options.isSkyHazed;
  // How the view draws the fog beyond the weather's keys: the engine's, or the enhanced fog and its strengths.
  const value: TRenderFogSettings = toLevelModeFeature("fog", settings, features);
  const isEnhanced: boolean = value.mode === ERenderFogMode.ENHANCED;

  return (
    <EditorPopoverToggle
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Fog"}
      description={
        isOn
          ? `Fog total at ${formatNumber(manual.fogDistance, 0)} m` + (isEnhanced ? ", thickened low down" : "")
          : "Fog off"
      }
      icon={<FoggyIcon />}
      isOn={isOn}
      toggleLabel={"Fog"}
      resetLabel={LEVEL_WEATHER_RESET_LABEL}
      onToggle={toggle}
      onReset={onWeatherReset}
    >
      <LevelManualWeatherVectorField field={"fogColor"} isColor manual={manual} onEdit={weatherService.editManual} />

      <LevelManualWeatherSlider
        field={"fogDistance"}
        manual={manual}
        format={(value: number) => `${formatNumber(value, 0)} m`}
        onEdit={weatherService.editManual}
      />

      <LevelManualWeatherSlider
        field={"fogDensity"}
        manual={manual}
        format={formatPercent}
        onEdit={weatherService.editManual}
      />

      <LevelManualWeatherSlider
        field={"farPlane"}
        manual={manual}
        format={(value: number) => `${formatNumber(value, 0)} m`}
        onEdit={weatherService.editManual}
      />

      <EditorPopoverGroupSection label={"Rendering"} resetLabel={LEVEL_SETTINGS_RESET_LABEL} onReset={onSettingsReset}>
        <RenderValueChoice
          label={"Fades into"}
          options={FADE_OPTIONS}
          value={isHazed ? "haze" : "sky"}
          onChange={(fade: "sky" | "haze") => viewService.setOption("isSkyHazed", fade === "haze")}
        />

        <RenderSettingEntries
          settings={RENDER_FOG_SETTINGS}
          names={["colour", "mode", "height", "density", "sunColor", "scattering", "skyFade"]}
          group={value}
          features={drawn}
          onSet={set}
        />
      </EditorPopoverGroupSection>
    </EditorPopoverToggle>
  );
}
