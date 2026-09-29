import { IRendererFeatureSettings } from "@xrf/renderer";
import { Nullable } from "@xrf/types";
import { ReactElement, ReactNode, useCallback } from "react";

import { LevelSunDescription, LevelWeatherTexture } from "@/core/ipc/types/xrf-app";
import { LevelAntialiasingAction } from "@/core/level/components/preview/LevelAntialiasingAction";
import { LevelCullingAction } from "@/core/level/components/preview/LevelCullingAction";
import { LevelFogAction } from "@/core/level/components/preview/LevelFogAction";
import { LevelGrassAction } from "@/core/level/components/preview/LevelGrassAction";
import { LevelLightsAction } from "@/core/level/components/preview/LevelLightsAction";
import { LevelOcclusionAction } from "@/core/level/components/preview/LevelOcclusionAction";
import { LevelOverlaysAction } from "@/core/level/components/preview/LevelOverlaysAction";
import { LevelRainAction } from "@/core/level/components/preview/LevelRainAction";
import { LevelShadowAction } from "@/core/level/components/preview/LevelShadowAction";
import { LevelSkyAction } from "@/core/level/components/preview/LevelSkyAction";
import { LevelSunAction } from "@/core/level/components/preview/LevelSunAction";
import { LevelSurfacesAction } from "@/core/level/components/preview/LevelSurfacesAction";
import { LevelWaterAction } from "@/core/level/components/preview/LevelWaterAction";
import { LevelWeatherAction } from "@/core/level/components/preview/LevelWeatherAction";
import { LevelWindAction } from "@/core/level/components/preview/LevelWindAction";
import { ILevelFeatureOptions, TLevelFeatureView } from "@/core/level/lib/features";
import { ILevelLodOptions } from "@/core/level/lib/lod/level-lod-options";
import { ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { ILevelManualWeather } from "@/core/level/lib/weather/level-manual-weather";
import { EditorToolbar } from "@/core/shell/editor/EditorToolbar";
import { EditorToolbarSeparator } from "@/core/shell/editor/EditorToolbarSeparator";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ILevelPreviewToolbarProps extends BaseComponentProps {
  subtitle?: ReactNode;
  options: ILevelViewOptions;
  /** How much the baked hemisphere darkens the ambient, which the baked light toggle carries. */
  hemiStrength: number;
  /** The keyframe on screen, which the sun, sky, clouds, fog, rain, water and wind popovers edit. */
  manual: ILevelManualWeather;
  /** Every sky the game's weather names, which the sky's popover offers. */
  skies: ReadonlyArray<LevelWeatherTexture>;
  /** Every clouds texture it names. */
  clouds: ReadonlyArray<LevelWeatherTexture>;
  /** The sun the open level was compiled against, which the sun's popover offers to light it from. */
  sun: Nullable<LevelSunDescription>;
  /** How far trees are drawn in full, which the impostors toggle carries. */
  lod: ILevelLodOptions;
  /** What the view sets over the settings' features for itself. */
  features: ILevelFeatureOptions;
  /** Each feature group as the view draws it while on, and whether the settings let it be on. */
  featureView: TLevelFeatureView;
  /** What the renderer's settings draw every viewport with, which the toggles can only narrow. */
  settings: IRendererFeatureSettings;
  /** Whether the settings time every viewport's passes. */
  isGpuTimed: boolean;
  /** Value pickers the surface contributes, drawn last, as every toolbar in this application orders them. */
  actions?: ReactNode;
  onChangeOptions: (options: ILevelViewOptions) => void;
  onChangeHemiStrength: (hemiStrength: number) => void;
  /** Sets keys of the keyframe set by hand, which lights the level from then on. */
  onEditManual: (patch: Partial<ILevelManualWeather>) => void;
  onChangeLod: (lod: ILevelLodOptions) => void;
  onChangeFeatures: (features: ILevelFeatureOptions) => void;
  /** Sets in the settings whether every viewport times its passes. */
  onChangeGpuTimed: (isGpuTimed: boolean) => void;
  onBack?: () => void;
}

/**
 * Composes the level view toggles in the editor toolbar. A toggle with settings behind it turns over on a click and
 * opens them on a right click; a group of toggles opens on a click.
 */
export function LevelPreviewToolbar({
  "data-testid": dataTestId,
  id,
  className,
  subtitle,
  options,
  hemiStrength,
  manual,
  skies,
  clouds,
  sun,
  lod,
  features,
  featureView,
  settings,
  isGpuTimed,
  actions,
  onChangeOptions,
  onChangeHemiStrength,
  onEditManual,
  onChangeLod,
  onChangeFeatures,
  onChangeGpuTimed,
  onBack,
}: ILevelPreviewToolbarProps): ReactElement {
  const onToggle = useCallback(
    (option: keyof ILevelViewOptions) => {
      onChangeOptions({ ...options, [option]: !options[option] });
    },
    [options, onChangeOptions]
  );

  return (
    <EditorToolbar
      data-testid={dataTestId}
      id={id}
      className={className}
      subtitle={subtitle}
      onBack={onBack}
      actions={
        <>
          <LevelSurfacesAction options={options} onToggle={onToggle} />

          <LevelCullingAction
            options={options}
            isOcclusionAvailable={settings.isOcclusionCulled}
            isImpostorsAvailable={settings.lod.isImpostors}
            lod={lod}
            onToggle={onToggle}
            onChangeLod={onChangeLod}
          />

          <LevelAntialiasingAction
            isOn={options.isAntialiased}
            settingsMode={settings.antialiasing}
            features={features}
            onToggle={() => onToggle("isAntialiased")}
            onChange={onChangeFeatures}
          />

          <EditorToolbarSeparator />

          <LevelWeatherAction />

          <LevelSunAction
            isOn={options.isSunVisible}
            manual={manual}
            sun={sun}
            onToggle={() => onToggle("isSunVisible")}
            onEdit={onEditManual}
          />

          <LevelLightsAction
            isOn={options.isLamplit}
            state={featureView.lights}
            features={features}
            onToggle={() => onToggle("isLamplit")}
            onChange={onChangeFeatures}
          />

          <LevelShadowAction
            isOn={options.isShadowed}
            state={featureView.shadows}
            features={features}
            onToggle={() => onToggle("isShadowed")}
            onChange={onChangeFeatures}
          />

          <LevelOcclusionAction
            options={options}
            state={featureView.ambientOcclusion}
            features={features}
            hemiStrength={hemiStrength}
            onToggle={onToggle}
            onChange={onChangeFeatures}
            onChangeHemiStrength={onChangeHemiStrength}
          />

          <LevelSkyAction
            options={options}
            manual={manual}
            skies={skies}
            clouds={clouds}
            onToggle={onToggle}
            onEdit={onEditManual}
          />

          <LevelFogAction
            isOn={options.isFogged}
            manual={manual}
            onToggle={() => onToggle("isFogged")}
            onEdit={onEditManual}
          />

          <LevelRainAction
            isOn={options.isRainy}
            manual={manual}
            onToggle={() => onToggle("isRainy")}
            onEdit={onEditManual}
          />

          <LevelWaterAction
            isOn={options.isWaterVisible}
            state={featureView.water}
            features={features}
            waterIntensity={manual.waterIntensity}
            onToggle={() => onToggle("isWaterVisible")}
            onChange={onChangeFeatures}
            onWaterIntensity={(waterIntensity: number) => onEditManual({ waterIntensity })}
          />

          <LevelGrassAction
            isOn={options.isGrassy}
            state={featureView.grass}
            features={features}
            onToggle={() => onToggle("isGrassy")}
            onChange={onChangeFeatures}
          />

          <LevelWindAction
            isOn={options.isWindy}
            manual={manual}
            onToggle={() => onToggle("isWindy")}
            onEdit={onEditManual}
          />

          <EditorToolbarSeparator />

          <LevelOverlaysAction
            options={options}
            isGpuTimed={isGpuTimed}
            onToggle={onToggle}
            onChangeGpuTimed={onChangeGpuTimed}
          />

          {actions ? (
            <>
              <EditorToolbarSeparator />
              {actions}
            </>
          ) : null}
        </>
      }
    />
  );
}
