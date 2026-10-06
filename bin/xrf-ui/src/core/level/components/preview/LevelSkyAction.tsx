import { default as NightsStayIcon } from "@mui/icons-material/NightsStay";
import { Button } from "@mui/material";
import { ReactElement, useCallback } from "react";

import { LevelWeatherTexture } from "@/core/ipc/types/xrf-app";
import { ERenderDebandingMode, RenderDebandingQuality } from "@/core/ipc/types/xrf-renderer";
import { LevelManualWeatherSlider } from "@/core/level/components/weather/LevelManualWeatherSlider";
import { LevelManualWeatherVectorField } from "@/core/level/components/weather/LevelManualWeatherVectorField";
import { LevelWeatherResetButton } from "@/core/level/components/weather/LevelWeatherResetButton";
import { LevelWeatherTextureField } from "@/core/level/components/weather/LevelWeatherTextureField";
import { ILevelFeatureOptions, TLevelDebandingOptions } from "@/core/level/lib/features";
import { ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { ILevelManualWeather } from "@/core/level/lib/weather/level-manual-weather";
import { RenderValueChoice } from "@/core/render/components/controls/RenderValueChoice";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import {
  describeRenderDebandingQuality,
  explainRenderDebandingMode,
  formatDebandingRadius,
  RENDER_DEBANDING_LIMITS,
  RENDER_DEBANDING_QUALITY_OPTIONS,
} from "@/core/render/lib/features";
import { TRenderDebandingSettings } from "@/core/render/lib/settings/render-feature-settings";
import { EditorPopoverGroup, EditorPopoverGroupSection } from "@/core/shell/editor/EditorPopoverGroup";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatDegrees } from "@/lib/format/angle";
import { formatPercent } from "@/lib/format/number";

/** The keys the popover sets. */
const SKY_KEYS: ReadonlyArray<keyof ILevelManualWeather> = [
  "skyTexture",
  "skyColor",
  "skyRotation",
  "cloudsTexture",
  "cloudsColor",
  "cloudsRotation",
];

interface ILevelSkyActionProps extends BaseComponentProps {
  options: ILevelViewOptions;
  /** The keyframe on screen: the one set by hand, or the weather's mix, which the first edit seeds it from. */
  manual: ILevelManualWeather;
  /** Every sky the game's weather names. */
  skies: ReadonlyArray<LevelWeatherTexture>;
  /** Every clouds texture it names. */
  clouds: ReadonlyArray<LevelWeatherTexture>;
  /** Whether and how the view smooths the sky's colour bands. */
  debanding: TRenderDebandingSettings;
  /** What the view sets over the settings, of which the debanding's part is changed. */
  features: ILevelFeatureOptions;
  onToggle: (option: keyof ILevelViewOptions) => void;
  onEdit: (patch: Partial<ILevelManualWeather>) => void;
  onChangeFeatures: (features: ILevelFeatureOptions) => void;
}

/**
 * The sky cube, its tint and turn, which its `#small` twin follows, the clouds over it with their cover, and whether
 * its colour bands are smoothed.
 */
export function LevelSkyAction({
  "data-testid": dataTestId = "level-sky-action",
  id,
  className,
  options,
  manual,
  skies,
  clouds,
  debanding,
  features,
  onToggle,
  onEdit,
  onChangeFeatures,
}: ILevelSkyActionProps): ReactElement {
  const { isSkyVisible, isClouded } = options;
  const isCloudy: boolean = isClouded && manual.cloudsTexture !== "";
  const isDebanded: boolean = debanding.mode === ERenderDebandingMode.ENHANCED;

  const setDebanding = useCallback(
    (part: Partial<TLevelDebandingOptions>): void =>
      onChangeFeatures({ ...features, debanding: { ...features.debanding, ...part } }),
    [features, onChangeFeatures]
  );

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
      <EditorPopoverGroupSection label={"Sky"} isOn={isSkyVisible} onToggle={() => onToggle("isSkyVisible")}>
        <LevelWeatherTextureField
          label={"sky_texture"}
          value={manual.skyTexture}
          textures={skies}
          onChange={(skyTexture: string) => onEdit({ skyTexture })}
        />

        <LevelManualWeatherVectorField field={"skyColor"} isColor manual={manual} onEdit={onEdit} />

        <LevelManualWeatherSlider field={"skyRotation"} manual={manual} format={formatDegrees} onEdit={onEdit} />
      </EditorPopoverGroupSection>

      <EditorPopoverGroupSection label={"Clouds"} isOn={isClouded} onToggle={() => onToggle("isClouded")}>
        <LevelWeatherTextureField
          label={"clouds_texture"}
          value={manual.cloudsTexture}
          textures={clouds}
          onChange={(cloudsTexture: string) => onEdit({ cloudsTexture })}
        />

        <LevelManualWeatherVectorField field={"cloudsColor"} isColor manual={manual} onEdit={onEdit} />

        <LevelManualWeatherSlider field={"cloudsRotation"} manual={manual} format={formatDegrees} onEdit={onEdit} />
      </EditorPopoverGroupSection>

      <LevelWeatherResetButton keys={SKY_KEYS} onEdit={onEdit} />

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

        <Button size={"small"} onClick={() => onChangeFeatures({ ...features, debanding: {} })}>
          Back to the settings for the debanding
        </Button>
      </EditorPopoverGroupSection>
    </EditorPopoverGroup>
  );
}
