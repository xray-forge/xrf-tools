import { ReactElement } from "react";

import { LevelCameraAction } from "@/core/level/components/preview/toolbar/camera/LevelCameraAction";
import { LevelGoToAction } from "@/core/level/components/preview/toolbar/camera/LevelGoToAction";
import { LevelCullingAction } from "@/core/level/components/preview/toolbar/contents/LevelCullingAction";
import { LevelGrassAction } from "@/core/level/components/preview/toolbar/contents/LevelGrassAction";
import { LevelParticlesAction } from "@/core/level/components/preview/toolbar/contents/LevelParticlesAction";
import { LevelSpawnAction } from "@/core/level/components/preview/toolbar/contents/LevelSpawnAction";
import { LevelWaterAction } from "@/core/level/components/preview/toolbar/contents/LevelWaterAction";
import { LevelFogAction } from "@/core/level/components/preview/toolbar/environment/LevelFogAction";
import { LevelRainAction } from "@/core/level/components/preview/toolbar/environment/LevelRainAction";
import { LevelSkyAction } from "@/core/level/components/preview/toolbar/environment/LevelSkyAction";
import { LevelWeatherAction } from "@/core/level/components/preview/toolbar/environment/LevelWeatherAction";
import { LevelWindAction } from "@/core/level/components/preview/toolbar/environment/LevelWindAction";
import { LevelImageQualityAction } from "@/core/level/components/preview/toolbar/image/LevelImageQualityAction";
import { LevelLookAction } from "@/core/level/components/preview/toolbar/image/LevelLookAction";
import { LevelLightsAction } from "@/core/level/components/preview/toolbar/lighting/LevelLightsAction";
import { LevelOcclusionAction } from "@/core/level/components/preview/toolbar/lighting/LevelOcclusionAction";
import { LevelReflectionsAction } from "@/core/level/components/preview/toolbar/lighting/LevelReflectionsAction";
import { LevelShadowAction } from "@/core/level/components/preview/toolbar/lighting/LevelShadowAction";
import { LevelSunAction } from "@/core/level/components/preview/toolbar/lighting/LevelSunAction";
import { LevelOverlaysAction } from "@/core/level/components/preview/toolbar/view/LevelOverlaysAction";
import { LevelShadingAction } from "@/core/level/components/preview/toolbar/view/LevelShadingAction";
import { RenderSliderReference } from "@/core/render/components/controls/RenderSliderReference";
import { ERenderSliderReferenceSource } from "@/core/render/lib/settings/render-slider-reference-source";
import { EditorToolbar } from "@/core/shell/editor/EditorToolbar";
import { EditorToolbarSeparator } from "@/core/shell/editor/EditorToolbarSeparator";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ILevelViewerToolbarProps extends BaseComponentProps {
  onBack?: () => void;
}

/**
 * Composes the level view toggles in the editor toolbar, by what each changes: the view, the image, the lighting, the
 * environment and the contents, then the camera's value pickers last. Each action reads and sets what it shows itself,
 * its sliders measured by the settings, which its popover resets to.
 */
export function LevelViewerToolbar({
  "data-testid": dataTestId,
  id,
  className,
  onBack,
}: ILevelViewerToolbarProps): ReactElement {
  return (
    <EditorToolbar
      data-testid={dataTestId}
      id={id}
      className={className}
      onBack={onBack}
      actions={
        <RenderSliderReference source={ERenderSliderReferenceSource.SETTINGS}>
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
        </RenderSliderReference>
      }
    />
  );
}
