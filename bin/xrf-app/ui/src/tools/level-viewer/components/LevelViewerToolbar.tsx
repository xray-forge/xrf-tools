import { EditorToolbarSeparator } from "@xrf/ui/editor/toolbar";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement } from "react";

import { LevelCameraAction } from "@/core/level/components/preview/toolbar/camera/LevelCameraAction";
import { LevelCameraEffectsAction } from "@/core/level/components/preview/toolbar/camera/LevelCameraEffectsAction";
import { LevelGoToAction } from "@/core/level/components/preview/toolbar/camera/LevelGoToAction";
import { LevelCullingAction } from "@/core/level/components/preview/toolbar/contents/LevelCullingAction";
import { LevelGrassAction } from "@/core/level/components/preview/toolbar/contents/LevelGrassAction";
import { LevelWaterAction } from "@/core/level/components/preview/toolbar/contents/LevelWaterAction";
import { LevelWeatherAction } from "@/core/level/components/preview/toolbar/environment/LevelWeatherAction";
import { LevelLightsAction } from "@/core/level/components/preview/toolbar/lighting/LevelLightsAction";
import { LevelSunAction } from "@/core/level/components/preview/toolbar/lighting/LevelSunAction";
import { LevelSoundAction } from "@/core/level/components/preview/toolbar/sound/LevelSoundAction";
import { LevelReliefAction } from "@/core/level/components/preview/toolbar/view/LevelReliefAction";
import { LevelShowAction } from "@/core/level/components/preview/toolbar/view/LevelShowAction";
import { RenderSliderReference } from "@/core/render/components/controls/RenderSliderReference";
import { ERenderSliderReferenceSource } from "@/core/render/lib/settings/choice/render-slider-reference-source";
import { EditorToolbar } from "@/core/shell/chrome/EditorToolbar";
import { LevelFogAction } from "@/tools/level-viewer/components/toolbar/environment/LevelFogAction";
import { LevelRainAction } from "@/tools/level-viewer/components/toolbar/environment/LevelRainAction";
import { LevelSkyAction } from "@/tools/level-viewer/components/toolbar/environment/LevelSkyAction";
import { LevelWindAction } from "@/tools/level-viewer/components/toolbar/environment/LevelWindAction";
import { LevelImageQualityAction } from "@/tools/level-viewer/components/toolbar/image/LevelImageQualityAction";
import { LevelLookAction } from "@/tools/level-viewer/components/toolbar/image/LevelLookAction";
import { LevelOcclusionAction } from "@/tools/level-viewer/components/toolbar/lighting/LevelOcclusionAction";
import { LevelReflectionsAction } from "@/tools/level-viewer/components/toolbar/lighting/LevelReflectionsAction";
import { LevelShadowAction } from "@/tools/level-viewer/components/toolbar/lighting/LevelShadowAction";

interface ILevelViewerToolbarProps extends BaseComponentProps {
  onBack?: () => void;
}

/**
 * Composes the level view toggles in the editor toolbar, by what each changes: the view, the image, the lighting, the
 * environment and the contents, then the camera's value pickers; the captures are the Actions menu's. Each action reads
 * and sets what it shows itself, its sliders measured by the settings, which its popover resets to.
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
          <LevelReliefAction />
          <LevelShowAction />

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
          <LevelSoundAction />

          <EditorToolbarSeparator />

          <LevelGrassAction />
          <LevelWaterAction />
          <LevelCullingAction />

          <EditorToolbarSeparator />

          <LevelCameraAction />
          <LevelCameraEffectsAction />
          <LevelGoToAction />
        </RenderSliderReference>
      }
    />
  );
}
