import { default as WaterDropIcon } from "@mui/icons-material/WaterDrop";
import { Button } from "@mui/material";
import { ReactElement, useCallback } from "react";

import { ThunderboltCollection } from "@/core/ipc/types/xrf-environment";
import { ERenderRainMode } from "@/core/ipc/types/xrf-renderer";
import { LevelManualWeatherSlider } from "@/core/level/components/weather/LevelManualWeatherSlider";
import { LevelManualWeatherVectorField } from "@/core/level/components/weather/LevelManualWeatherVectorField";
import { LevelWeatherCollectionSelect } from "@/core/level/components/weather/LevelWeatherCollectionSelect";
import { LevelWeatherResetButton } from "@/core/level/components/weather/LevelWeatherResetButton";
import { ILevelFeatureOptions, TLevelRainOptions } from "@/core/level/lib/features";
import { ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { ILevelManualWeather } from "@/core/level/lib/weather/level-manual-weather";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import {
  explainRenderRainMode,
  formatGrassRadius,
  formatRainShare,
  RENDER_RAIN_LIMITS,
} from "@/core/render/lib/features";
import { TRenderRainSettings } from "@/core/render/lib/settings/render-feature-settings";
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
  /** The rain's wetting as the view draws it: whether puddles gather, and their strengths. */
  puddles: TRenderRainSettings;
  /** What the view sets over the settings, of which the rain's part is changed. */
  features: ILevelFeatureOptions;
  onToggle: (option: keyof ILevelViewOptions) => void;
  onEdit: (patch: Partial<ILevelManualWeather>) => void;
  onChangeFeatures: (features: ILevelFeatureOptions) => void;
}

/**
 * How hard it rains and the streaks' colour, the bolts that strike with it, and whether puddles gather on flat ground
 * over the engine's own wetting: how many of the sites hold one, how much they reflect, how far they are drawn and how
 * strongly the rain ripples them; whether any of it happens at all.
 */
export function LevelRainAction({
  "data-testid": dataTestId = "level-rain-action",
  id,
  className,
  options,
  manual,
  collections,
  puddles,
  features,
  onToggle,
  onEdit,
  onChangeFeatures,
}: ILevelRainActionProps): ReactElement {
  const { isRainy, isThundering } = options;
  const isRaining: boolean = isRainy && manual.rainDensity > 0;
  const isStriking: boolean = isThundering && manual.thunderboltCollection !== "";
  const isPuddled: boolean = puddles.mode === ERenderRainMode.ENHANCED;

  const setPuddles = useCallback(
    (part: Partial<TLevelRainOptions>): void => onChangeFeatures({ ...features, rain: { ...features.rain, ...part } }),
    [features, onChangeFeatures]
  );

  return (
    <EditorPopoverGroup
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Rain"}
      description={[
        isRaining ? `Rain at ${formatPercent(manual.rainDensity)}` : isRainy ? "No rain" : "Rain off",
        isStriking ? `bolts of ${manual.thunderboltCollection}` : isThundering ? "no thunder" : "thunder off",
        isPuddled ? `puddles at ${formatRainShare(puddles.puddles)} of the sites` : "no puddles",
      ].join(", ")}
      icon={<WaterDropIcon />}
      isActive={isRaining || isStriking || isPuddled}
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

      <EditorPopoverGroupSection
        label={"Puddles"}
        description={explainRenderRainMode(puddles.mode)}
        isOn={isPuddled}
        onToggle={() => setPuddles({ mode: isPuddled ? ERenderRainMode.ENGINE : ERenderRainMode.ENHANCED })}
      >
        {isPuddled ? (
          <>
            <RenderValueSlider
              label={"Puddles"}
              value={puddles.puddles}
              {...RENDER_RAIN_LIMITS.puddles}
              format={formatRainShare}
              onChange={(share: number) => setPuddles({ puddles: share })}
            />

            <RenderValueSlider
              label={"Reflectivity"}
              value={puddles.reflectivity}
              {...RENDER_RAIN_LIMITS.reflectivity}
              format={formatRainShare}
              onChange={(reflectivity: number) => setPuddles({ reflectivity })}
            />

            <RenderValueSlider
              label={"Distance"}
              value={puddles.distance}
              {...RENDER_RAIN_LIMITS.distance}
              format={formatGrassRadius}
              onChange={(distance: number) => setPuddles({ distance })}
            />

            <RenderValueSlider
              label={"Ripples"}
              value={puddles.ripples}
              {...RENDER_RAIN_LIMITS.ripples}
              format={formatRainShare}
              onChange={(ripples: number) => setPuddles({ ripples })}
            />
          </>
        ) : null}

        <Button size={"small"} onClick={() => onChangeFeatures({ ...features, rain: {} })}>
          Back to the settings for the puddles
        </Button>
      </EditorPopoverGroupSection>
    </EditorPopoverGroup>
  );
}
