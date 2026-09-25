import { default as GridOnIcon } from "@mui/icons-material/GridOn";
import { default as HexagonIcon } from "@mui/icons-material/Hexagon";
import { default as TextureIcon } from "@mui/icons-material/Texture";
import { default as ThreeDRotationIcon } from "@mui/icons-material/ThreeDRotation";
import { IRendererFeatureSettings } from "@xrf/renderer";
import { ReactElement, ReactNode, useCallback } from "react";

import { LevelAmbientOcclusionAction } from "@/core/level/components/preview/LevelAmbientOcclusionAction";
import { LevelAntialiasingAction } from "@/core/level/components/preview/LevelAntialiasingAction";
import { LevelBakedAction } from "@/core/level/components/preview/LevelBakedAction";
import { LevelFogAction } from "@/core/level/components/preview/LevelFogAction";
import { LevelGrassAction } from "@/core/level/components/preview/LevelGrassAction";
import { LevelLodAction } from "@/core/level/components/preview/LevelLodAction";
import { LevelReadoutAction } from "@/core/level/components/preview/LevelReadoutAction";
import { LevelShadowAction } from "@/core/level/components/preview/LevelShadowAction";
import { LevelSunAction } from "@/core/level/components/preview/LevelSunAction";
import { LevelWindAction } from "@/core/level/components/preview/LevelWindAction";
import {
  ILevelFeatureOptions,
  toLevelRendererAmbientOcclusion,
  toLevelRendererGrassSettings,
  toLevelRendererShadows,
} from "@/core/level/lib/features/level-feature-options";
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
  /** How far trees are drawn in full, which the impostors toggle carries. */
  lod: ILevelLodOptions;
  /** What the view draws its shadows and antialiasing with, over the settings. */
  features: ILevelFeatureOptions;
  /** What the renderer's settings draw every viewport with, which the toggles can only narrow. */
  settings: IRendererFeatureSettings;
  /** Value pickers the surface contributes, drawn last, as every toolbar in this application orders them. */
  actions?: ReactNode;
  onChangeOptions: (options: ILevelViewOptions) => void;
  onChangeLighting: (lighting: ILevelLighting) => void;
  onChangeLod: (lod: ILevelLodOptions) => void;
  onChangeFeatures: (features: ILevelFeatureOptions) => void;
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
  lod,
  features,
  settings,
  actions,
  onChangeOptions,
  onChangeLighting,
  onChangeLod,
  onChangeFeatures,
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

          <EditorToolbarSeparator />

          <LevelBakedAction
            isOn={options.isLit}
            lighting={lighting}
            onToggle={() => onToggle("isLit")}
            onChange={onChangeLighting}
          />

          <LevelSunAction
            isOn={options.isSunVisible}
            lighting={lighting}
            onToggle={() => onToggle("isSunVisible")}
            onChange={onChangeLighting}
          />

          <LevelShadowAction
            isOn={options.isShadowed}
            isAvailable={settings.shadows.isEnabled}
            shadows={toLevelRendererShadows(settings.shadows, features, true)}
            features={features}
            onToggle={() => onToggle("isShadowed")}
            onChange={onChangeFeatures}
          />

          <LevelAmbientOcclusionAction
            isOn={options.isOccluded}
            isAvailable={settings.ambientOcclusion.isEnabled}
            occlusion={toLevelRendererAmbientOcclusion(settings.ambientOcclusion, features, true)}
            features={features}
            onToggle={() => onToggle("isOccluded")}
            onChange={onChangeFeatures}
          />

          <LevelFogAction
            isOn={options.isFogged}
            lighting={lighting}
            onToggle={() => onToggle("isFogged")}
            onChange={onChangeLighting}
          />

          <LevelGrassAction
            isOn={options.isGrassy}
            isAvailable={settings.grass.isEnabled}
            grass={toLevelRendererGrassSettings(settings.grass, features, true)}
            features={features}
            onToggle={() => onToggle("isGrassy")}
            onChange={onChangeFeatures}
          />

          <LevelWindAction
            isOn={options.isWindy}
            lighting={lighting}
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
            isAdvanced={options.isAdvancedStatsVisible}
            onToggle={() => onToggle("isStatsVisible")}
            onToggleAdvanced={() => onToggle("isAdvancedStatsVisible")}
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
