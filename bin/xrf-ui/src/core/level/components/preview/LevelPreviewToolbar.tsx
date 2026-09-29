import { default as GridOnIcon } from "@mui/icons-material/GridOn";
import { default as HexagonIcon } from "@mui/icons-material/Hexagon";
import { default as LayersIcon } from "@mui/icons-material/Layers";
import { default as TerrainIcon } from "@mui/icons-material/Terrain";
import { default as TextureIcon } from "@mui/icons-material/Texture";
import { default as ThreeDRotationIcon } from "@mui/icons-material/ThreeDRotation";
import { ERendererRenderScale, IRendererFeatureSettings } from "@xrf/renderer";
import { Nullable } from "@xrf/types";
import { ReactElement, ReactNode, useCallback } from "react";

import { LevelSunDescription } from "@/core/ipc/types/xrf-app";
import { LevelAmbientOcclusionAction } from "@/core/level/components/preview/LevelAmbientOcclusionAction";
import { LevelAntialiasingAction } from "@/core/level/components/preview/LevelAntialiasingAction";
import { LevelBakedAction } from "@/core/level/components/preview/LevelBakedAction";
import { LevelFogAction } from "@/core/level/components/preview/LevelFogAction";
import { LevelGrassAction } from "@/core/level/components/preview/LevelGrassAction";
import { LevelLightsAction } from "@/core/level/components/preview/LevelLightsAction";
import { LevelLodAction } from "@/core/level/components/preview/LevelLodAction";
import { LevelReadoutAction } from "@/core/level/components/preview/LevelReadoutAction";
import { LevelRenderScaleAction } from "@/core/level/components/preview/LevelRenderScaleAction";
import { LevelShadowAction } from "@/core/level/components/preview/LevelShadowAction";
import { LevelSunAction } from "@/core/level/components/preview/LevelSunAction";
import { LevelWaterAction } from "@/core/level/components/preview/LevelWaterAction";
import { LevelWindAction } from "@/core/level/components/preview/LevelWindAction";
import { ILevelFeatureOptions, TLevelFeatureView } from "@/core/level/lib/features";
import { ILevelLighting } from "@/core/level/lib/lighting/level-lighting";
import { ILevelLodOptions } from "@/core/level/lib/lod/level-lod-options";
import { ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { EditorToolbar } from "@/core/shell/editor/EditorToolbar";
import { EditorToolbarSeparator } from "@/core/shell/editor/EditorToolbarSeparator";
import { EditorViewToggle } from "@/core/shell/editor/EditorViewToggle";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ILevelPreviewToolbarProps extends BaseComponentProps {
  subtitle?: ReactNode;
  options: ILevelViewOptions;
  /** What the level is lit and fogged with, which the light and fog toggles carry the settings of. */
  lighting: ILevelLighting;
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
  /** Why the sun, fog and wind settings do nothing now, or null while they light the level. */
  lightingLock?: Nullable<string>;
  /** Value pickers the surface contributes, drawn last, as every toolbar in this application orders them. */
  actions?: ReactNode;
  onChangeOptions: (options: ILevelViewOptions) => void;
  onChangeLighting: (lighting: ILevelLighting) => void;
  onChangeLod: (lod: ILevelLodOptions) => void;
  onChangeFeatures: (features: ILevelFeatureOptions) => void;
  /** Sets the render scale in the settings, every viewport's. */
  onChangeScale: (scale: ERendererRenderScale) => void;
  /** Sets in the settings whether every viewport times its passes. */
  onChangeGpuTimed: (isGpuTimed: boolean) => void;
  onBack?: () => void;
}

/**
 * Composes the level view toggles in the editor toolbar. A toggle with settings behind it turns over on a click and
 * opens them on a right click.
 */
export function LevelPreviewToolbar({
  "data-testid": dataTestId,
  id,
  className,
  subtitle,
  options,
  lighting,
  sun,
  lod,
  features,
  featureView,
  settings,
  isGpuTimed,
  lightingLock = null,
  actions,
  onChangeOptions,
  onChangeLighting,
  onChangeLod,
  onChangeFeatures,
  onChangeScale,
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
          <EditorViewToggle
            label={"Wireframe"}
            icon={<HexagonIcon />}
            isOn={options.isWireframe}
            onToggle={() => onToggle("isWireframe")}
          />

          <EditorViewToggle
            label={"Textures"}
            icon={<TextureIcon />}
            isOn={options.isTextured}
            onToggle={() => onToggle("isTextured")}
          />

          <EditorViewToggle
            label={"Bumps"}
            description={
              options.isBumped
                ? "Surfaces shade with the bump pairs their textures declare"
                : "Every surface shades flat, as though no pair were bound"
            }
            icon={<TerrainIcon />}
            isOn={options.isBumped}
            onToggle={() => onToggle("isBumped")}
          />

          <EditorViewToggle
            label={"Occlusion culling"}
            description={
              options.isOcclusionCulled
                ? "What the depth hides is culled before it draws"
                : "Every static draw the frustum keeps is drawn, hidden or not"
            }
            unavailableTitle={"Occlusion culling is off in Settings, under Rendering"}
            icon={<LayersIcon />}
            isOn={options.isOcclusionCulled && settings.isOcclusionCulled}
            isDisabled={!settings.isOcclusionCulled}
            onToggle={() => onToggle("isOcclusionCulled")}
          />

          <LevelLodAction
            isOn={options.isImpostors}
            isAvailable={settings.lod.isImpostors}
            lod={lod}
            onToggle={() => onToggle("isImpostors")}
            onChange={onChangeLod}
          />

          <LevelAntialiasingAction
            isOn={options.isAntialiased}
            settingsMode={settings.antialiasing}
            features={features}
            onToggle={() => onToggle("isAntialiased")}
            onChange={onChangeFeatures}
          />

          <LevelRenderScaleAction scale={settings.upscaling.scale} onChange={onChangeScale} />

          <EditorToolbarSeparator />

          <LevelBakedAction
            isOn={options.isBaked}
            lighting={lighting}
            onToggle={() => onToggle("isBaked")}
            onChange={onChangeLighting}
          />

          <LevelSunAction
            isOn={options.isSunVisible}
            lighting={lighting}
            sun={sun}
            lockedReason={lightingLock}
            onToggle={() => onToggle("isSunVisible")}
            onChange={onChangeLighting}
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

          <LevelAmbientOcclusionAction
            isOn={options.isOccluded}
            state={featureView.ambientOcclusion}
            features={features}
            onToggle={() => onToggle("isOccluded")}
            onChange={onChangeFeatures}
          />

          <LevelFogAction
            isOn={options.isFogged}
            lighting={lighting}
            lockedReason={lightingLock}
            onToggle={() => onToggle("isFogged")}
            onChange={onChangeLighting}
          />

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
            onToggle={() => onToggle("isWaterVisible")}
            onChange={onChangeFeatures}
          />

          <LevelWindAction
            isOn={options.isWindy}
            lighting={lighting}
            lockedReason={lightingLock}
            onToggle={() => onToggle("isWindy")}
            onChange={onChangeLighting}
          />

          <EditorToolbarSeparator />

          <EditorViewToggle
            label={"Grid"}
            icon={<GridOnIcon />}
            isOn={options.isGridVisible}
            onToggle={() => onToggle("isGridVisible")}
          />

          <EditorViewToggle
            label={"Axes"}
            icon={<ThreeDRotationIcon />}
            isOn={options.isAxesVisible}
            onToggle={() => onToggle("isAxesVisible")}
          />

          <LevelReadoutAction
            isOn={options.isStatsVisible}
            isGpuTimed={isGpuTimed}
            onToggle={() => onToggle("isStatsVisible")}
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
