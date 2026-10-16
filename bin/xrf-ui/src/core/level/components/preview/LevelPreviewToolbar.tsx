import { ReactElement, ReactNode } from "react";

import { LevelCameraAction } from "@/core/level/components/preview/LevelCameraAction";
import { LevelCullingAction } from "@/core/level/components/preview/LevelCullingAction";
import { LevelFogAction } from "@/core/level/components/preview/LevelFogAction";
import { LevelGoToAction } from "@/core/level/components/preview/LevelGoToAction";
import { LevelGrassAction } from "@/core/level/components/preview/LevelGrassAction";
import { LevelImageQualityAction } from "@/core/level/components/preview/LevelImageQualityAction";
import { LevelLightsAction } from "@/core/level/components/preview/LevelLightsAction";
import { LevelLookAction } from "@/core/level/components/preview/LevelLookAction";
import { LevelOcclusionAction } from "@/core/level/components/preview/LevelOcclusionAction";
import { LevelOverlaysAction } from "@/core/level/components/preview/LevelOverlaysAction";
import { LevelParticlesAction } from "@/core/level/components/preview/LevelParticlesAction";
import { LevelRainAction } from "@/core/level/components/preview/LevelRainAction";
import { LevelReflectionsAction } from "@/core/level/components/preview/LevelReflectionsAction";
import { LevelShadingAction } from "@/core/level/components/preview/LevelShadingAction";
import { LevelShadowAction } from "@/core/level/components/preview/LevelShadowAction";
import { LevelSkyAction } from "@/core/level/components/preview/LevelSkyAction";
import { LevelSpawnAction } from "@/core/level/components/preview/LevelSpawnAction";
import { LevelSunAction } from "@/core/level/components/preview/LevelSunAction";
import { LevelWaterAction } from "@/core/level/components/preview/LevelWaterAction";
import { LevelWeatherAction } from "@/core/level/components/preview/LevelWeatherAction";
import { LevelWindAction } from "@/core/level/components/preview/LevelWindAction";
import { EditorToolbar } from "@/core/shell/editor/EditorToolbar";
import { EditorToolbarSeparator } from "@/core/shell/editor/EditorToolbarSeparator";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ILevelPreviewToolbarProps extends BaseComponentProps {
  subtitle?: ReactNode;
  onBack?: () => void;
}

/**
 * Composes the level view toggles in the editor toolbar, by what each changes: the view, the image, the lighting, the
 * environment and the contents, then the camera's value pickers last. Each action reads and sets what it shows itself.
 */
export function LevelPreviewToolbar({
  "data-testid": dataTestId,
  id,
  className,
  subtitle,
  onBack,
}: ILevelPreviewToolbarProps): ReactElement {
  return (
    <EditorToolbar
      data-testid={dataTestId}
      id={id}
      className={className}
      subtitle={subtitle}
      onBack={onBack}
      actions={
        <>
          <LevelShadingAction />
          <LevelOverlaysAction />

          <EditorToolbarSeparator />

          <LevelLookAction />
          <LevelImageQualityAction />

          <EditorToolbarSeparator />

          <LevelSunAction />
          <LevelLightsAction />
          <LevelShadowAction />
          <LevelOcclusionAction />
          <LevelReflectionsAction />

          <EditorToolbarSeparator />

          <LevelWeatherAction />
          <LevelSkyAction />
          <LevelFogAction />
          <LevelRainAction />
          <LevelWindAction />

          <EditorToolbarSeparator />

          <LevelSpawnAction />
          <LevelParticlesAction />
          <LevelGrassAction />
          <LevelWaterAction />
          <LevelCullingAction />

          <EditorToolbarSeparator />

          <LevelCameraAction />
          <LevelGoToAction />
        </>
      }
    />
  );
}
