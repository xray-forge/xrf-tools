import { useInjection } from "@wirestate/react";
import { Optional } from "@xrf/types";
import { EditorIconAction } from "@xrf/ui/editor/icon-action";
import { EditorPopoverGroup } from "@xrf/ui/editor/popover-group";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { CameraOutlinedIcon } from "@xrf/ui/icons/camera-outlined";
import { CenterFocusStrongIcon } from "@xrf/ui/icons/center-focus-strong";
import { ReactElement } from "react";

import { useLevelFeatureOverride } from "@/core/level/components/preview/toolbar/use-level-feature-override";
import {
  LEVEL_SETTINGS_RESET_LABEL,
  useLevelFeatureReset,
} from "@/core/level/components/preview/toolbar/use-level-feature-reset";
import { useLevelRenderFeatures } from "@/core/level/components/preview/toolbar/use-level-render-features";
import { toLevelModeFeature } from "@/core/level/lib/features/level-feature-options";
import { LevelViewportService } from "@/core/level/services/level-viewport";
import { RenderSettingSection } from "@/core/render/components/settings/RenderSettingSection";
import {
  formatCameraFocusDistance,
  RENDER_CAMERA_SETTINGS,
} from "@/core/render/lib/features/image/render-camera-choices";
import { TRenderCameraSettings } from "@/core/render/lib/settings/choice/render-feature-settings";

/**
 * The camera this view's frame is shown through: its motion blur, its depth of field focused by a value or by a click
 * in the view, its vignette, its film grain and its chromatic aberration, a section each, all over the settings for the
 * view alone and set back together from the popover's heading.
 */
export function LevelCameraEffectsAction({
  "data-testid": dataTestId = "level-camera-effects-action",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const viewportService: LevelViewportService = useInjection(LevelViewportService);
  const { settings, features, drawn } = useLevelRenderFeatures();
  const { set } = useLevelFeatureOverride("camera");
  const onReset: Optional<() => void> = useLevelFeatureReset("camera");

  const camera: TRenderCameraSettings = toLevelModeFeature("camera", settings, features);
  const isFocusPicking: boolean = viewportService.isFocusPicking;
  const effects: Array<string> = [
    camera.isMotionBlurred ? "motion blur" : null,
    camera.isDefocused ? `focused at ${formatCameraFocusDistance(camera.focusDistance)}` : null,
    camera.isVignetted ? "vignette" : null,
    camera.isGrained ? "grain" : null,
    camera.isAberrated ? "aberration" : null,
  ].filter((it): it is string => it !== null);
  const summary: string = effects.join(", ");

  return (
    <EditorPopoverGroup
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Camera effects"}
      description={summary ? summary.charAt(0).toUpperCase() + summary.slice(1) : "None"}
      icon={<CameraOutlinedIcon />}
      isActive={effects.length > 0}
      resetLabel={LEVEL_SETTINGS_RESET_LABEL}
      onReset={onReset}
    >
      <RenderSettingSection
        settings={RENDER_CAMERA_SETTINGS}
        name={"isMotionBlurred"}
        names={["shutter"]}
        group={camera}
        features={drawn}
        onSet={set}
      />

      <RenderSettingSection
        data-testid={"level-camera-depth-of-field"}
        settings={RENDER_CAMERA_SETTINGS}
        name={"isDefocused"}
        names={["fStop", "focusDistance"]}
        group={camera}
        features={drawn}
        actions={
          <EditorIconAction
            aria-pressed={isFocusPicking}
            label={"Focus by a click"}
            description={
              isFocusPicking ? "Click in the view to focus there" : "Focus where the next click in the view lands"
            }
            icon={<CenterFocusStrongIcon />}
            isActive={isFocusPicking}
            onClick={() => viewportService.setFocusPicking(!isFocusPicking)}
          />
        }
        onSet={set}
      />

      <RenderSettingSection
        settings={RENDER_CAMERA_SETTINGS}
        name={"isVignetted"}
        names={["vignetteStrength"]}
        group={camera}
        features={drawn}
        onSet={set}
      />

      <RenderSettingSection
        settings={RENDER_CAMERA_SETTINGS}
        name={"isGrained"}
        names={["grainStrength", "grainSize"]}
        group={camera}
        features={drawn}
        onSet={set}
      />

      <RenderSettingSection
        settings={RENDER_CAMERA_SETTINGS}
        name={"isAberrated"}
        names={["aberrationStrength"]}
        group={camera}
        features={drawn}
        onSet={set}
      />
    </EditorPopoverGroup>
  );
}
