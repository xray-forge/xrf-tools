import { Nullable } from "@xrf/types";
import { ReactElement, ReactNode, useCallback } from "react";

import { LevelSunDescription, LevelWeatherTexture } from "@/core/ipc/types/xrf-app";
import { ThunderboltCollection } from "@/core/ipc/types/xrf-environment";
import { LevelAntialiasingAction } from "@/core/level/components/preview/LevelAntialiasingAction";
import { LevelCullingAction } from "@/core/level/components/preview/LevelCullingAction";
import { LevelFogAction } from "@/core/level/components/preview/LevelFogAction";
import { LevelGrassAction } from "@/core/level/components/preview/LevelGrassAction";
import { LevelLightsAction } from "@/core/level/components/preview/LevelLightsAction";
import { LevelLookAction } from "@/core/level/components/preview/LevelLookAction";
import { LevelOcclusionAction } from "@/core/level/components/preview/LevelOcclusionAction";
import { LevelOverlaysAction } from "@/core/level/components/preview/LevelOverlaysAction";
import { LevelRainAction } from "@/core/level/components/preview/LevelRainAction";
import { LevelShadingAction } from "@/core/level/components/preview/LevelShadingAction";
import { LevelShadowAction } from "@/core/level/components/preview/LevelShadowAction";
import { LevelSkyAction } from "@/core/level/components/preview/LevelSkyAction";
import { LevelSpawnAction } from "@/core/level/components/preview/LevelSpawnAction";
import { LevelSunAction } from "@/core/level/components/preview/LevelSunAction";
import { LevelWaterAction } from "@/core/level/components/preview/LevelWaterAction";
import { LevelWeatherAction } from "@/core/level/components/preview/LevelWeatherAction";
import { LevelWindAction } from "@/core/level/components/preview/LevelWindAction";
import { ILevelFeatureOptions, TLevelFeatureView } from "@/core/level/lib/features";
import { ILevelLodOptions } from "@/core/level/lib/lod/level-lod-options";
import { ELevelShading } from "@/core/level/lib/view/level-shading";
import { ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { ILevelManualWeather } from "@/core/level/lib/weather/level-manual-weather";
import { ILevelSunShaftsOptions } from "@/core/level/lib/weather/level-sun-shafts-options";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/render-feature-settings";
import { EditorToolbar } from "@/core/shell/editor/EditorToolbar";
import { EditorToolbarSeparator } from "@/core/shell/editor/EditorToolbarSeparator";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ILevelPreviewToolbarProps extends BaseComponentProps {
  subtitle?: ReactNode;
  options: ILevelViewOptions;
  /** How much the baked hemisphere darkens the ambient, which the baked light toggle carries. */
  hemiStrength: number;
  /** The keyframe on screen, which the sun, sky, clouds, fog, rain, thunder, water and wind popovers edit. */
  manual: ILevelManualWeather;
  /** Every sky the game's weather names, which the sky's popover offers. */
  skies: ReadonlyArray<LevelWeatherTexture>;
  /** Every clouds texture it names. */
  clouds: ReadonlyArray<LevelWeatherTexture>;
  /** Every thunderbolt collection of the game, which the rain's popover offers. */
  collections: ReadonlyArray<ThunderboltCollection>;
  /** The sun the open level was compiled against, which the sun's popover offers to light it from. */
  sun: Nullable<LevelSunDescription>;
  /** The sun or moon the sky draws now, by its `suns.ltx` section, which lights the sun's button; null for neither. */
  drawnSun: Nullable<string>;
  /** Every lens flare of the game, which the sun's popover offers. */
  suns: ReadonlyArray<string>;
  /** How the level's sun shafts step, and the floor under their density, which the sun's popover sets. */
  sunShafts: ILevelSunShaftsOptions;
  /** How far trees are drawn in full, which the impostors toggle carries. */
  lod: ILevelLodOptions;
  /** What the view sets over the settings' features for itself. */
  features: ILevelFeatureOptions;
  /** Each feature group as the view draws it while on, and whether the settings let it be on. */
  featureView: TLevelFeatureView;
  /** What the renderer's settings draw every viewport with, which the toggles can only narrow. */
  settings: IRenderFeatureSettings;
  /** Whether the settings time every viewport's passes. */
  isGpuTimed: boolean;
  /** What the viewport shows of its surfaces. */
  shading: ELevelShading;
  /** Value pickers the surface contributes, drawn last, as every toolbar in this application orders them. */
  actions?: ReactNode;
  onChangeOptions: (options: ILevelViewOptions) => void;
  onChangeHemiStrength: (hemiStrength: number) => void;
  onChangeSunShafts: (sunShafts: ILevelSunShaftsOptions) => void;
  /** Sets keys of the keyframe set by hand, which lights the level from then on. */
  onEditManual: (patch: Partial<ILevelManualWeather>) => void;
  onChangeLod: (lod: ILevelLodOptions) => void;
  onChangeFeatures: (features: ILevelFeatureOptions) => void;
  /** Sets in the settings whether every viewport times its passes. */
  onChangeGpuTimed: (isGpuTimed: boolean) => void;
  onChangeShading: (shading: ELevelShading) => void;
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
  collections,
  sun,
  drawnSun,
  suns,
  sunShafts,
  lod,
  features,
  featureView,
  settings,
  isGpuTimed,
  shading,
  actions,
  onChangeOptions,
  onChangeHemiStrength,
  onChangeSunShafts,
  onEditManual,
  onChangeLod,
  onChangeFeatures,
  onChangeGpuTimed,
  onChangeShading,
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
          <LevelShadingAction
            options={options}
            shading={shading}
            onToggle={onToggle}
            onChangeShading={onChangeShading}
          />

          <LevelOverlaysAction
            options={options}
            isGpuTimed={isGpuTimed}
            onToggle={onToggle}
            onChangeGpuTimed={onChangeGpuTimed}
          />

          <EditorToolbarSeparator />

          <LevelLookAction />

          <LevelAntialiasingAction
            isOn={options.isAntialiased}
            settingsMode={settings.antialiasing}
            features={features}
            onToggle={() => onToggle("isAntialiased")}
            onChange={onChangeFeatures}
          />

          <EditorToolbarSeparator />

          <LevelSunAction
            manual={manual}
            sun={sun}
            drawnSun={drawnSun}
            suns={suns}
            options={options}
            sunShafts={sunShafts}
            onToggle={onToggle}
            onChangeSunShafts={onChangeSunShafts}
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

          <EditorToolbarSeparator />

          <LevelWeatherAction />

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
            isHazed={options.isSkyHazed}
            onToggle={() => onToggle("isFogged")}
            onEdit={onEditManual}
            onHazed={(isSkyHazed: boolean) => onChangeOptions({ ...options, isSkyHazed })}
          />

          <LevelRainAction
            options={options}
            manual={manual}
            collections={collections}
            onToggle={onToggle}
            onEdit={onEditManual}
          />

          <LevelWindAction
            isOn={options.isWindy}
            manual={manual}
            onToggle={() => onToggle("isWindy")}
            onEdit={onEditManual}
          />

          <EditorToolbarSeparator />

          <LevelSpawnAction options={options} onToggle={onToggle} />

          <LevelGrassAction
            isOn={options.isGrassy}
            state={featureView.grass}
            features={features}
            onToggle={() => onToggle("isGrassy")}
            onChange={onChangeFeatures}
          />

          <LevelWaterAction
            isOn={options.isWaterVisible}
            state={featureView.water}
            features={features}
            manual={manual}
            onToggle={() => onToggle("isWaterVisible")}
            onChange={onChangeFeatures}
            onEdit={onEditManual}
          />

          <LevelCullingAction
            options={options}
            isOcclusionAvailable={settings.isOcclusionCulled}
            isImpostorsAvailable={settings.lod.isImpostors}
            lod={lod}
            onToggle={onToggle}
            onChangeLod={onChangeLod}
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
