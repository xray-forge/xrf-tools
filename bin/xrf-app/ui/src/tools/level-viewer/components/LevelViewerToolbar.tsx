import { EditorToolbarSeparator } from "@xrf/ui/editor/toolbar";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement } from "react";

import { LevelWeatherAction } from "@/core/level/components/preview/toolbar/environment/LevelWeatherAction";
import { LevelCameraEffectsAction } from "@/core/level/components/preview/toolbar/image/LevelCameraEffectsAction";
import { LevelLightsAction } from "@/core/level/components/preview/toolbar/lighting/LevelLightsAction";
import { LevelSunAction } from "@/core/level/components/preview/toolbar/lighting/LevelSunAction";
import { LevelSoundAction } from "@/core/level/components/preview/toolbar/sound/LevelSoundAction";
import { LevelCameraAction } from "@/core/level/components/preview/toolbar/view/LevelCameraAction";
import { LevelShowAction } from "@/core/level/components/preview/toolbar/view/LevelShowAction";
import { LevelCullingAction } from "@/core/level/components/preview/toolbar/world/LevelCullingAction";
import { LevelGrassAction } from "@/core/level/components/preview/toolbar/world/LevelGrassAction";
import { LevelReliefAction } from "@/core/level/components/preview/toolbar/world/LevelReliefAction";
import { LevelWaterAction } from "@/core/level/components/preview/toolbar/world/LevelWaterAction";
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
 * The level view's toolbar actions, in the order a look at a level is made: viewport, world, lighting, environment,
 * image, then sound. Captures live in the Actions menu.
 *
 * Each action owns its state; its sliders are measured against the settings, which its popover resets to.
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
          <LevelShowAction />
          <LevelCameraAction />

          <EditorToolbarSeparator />

          <LevelReliefAction />
          <LevelGrassAction />
          <LevelWaterAction />
          <LevelCullingAction />

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

          <LevelLookAction />
          <LevelCameraEffectsAction />
          <LevelImageQualityAction />

          <EditorToolbarSeparator />

          <LevelSoundAction />
        </RenderSliderReference>
      }
    />
  );
}
