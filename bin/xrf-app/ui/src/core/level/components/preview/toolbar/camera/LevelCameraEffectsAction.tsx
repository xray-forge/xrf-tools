import { useInjection } from "@wirestate/react";
import { EditorPopoverGroup } from "@xrf/ui/editor/popover-group";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { CameraOutlinedIcon } from "@xrf/ui/icons/camera-outlined";
import { CenterFocusStrongIcon } from "@xrf/ui/icons/center-focus-strong";
import { Button } from "@xrf/ui/material/button";
import { ReactElement } from "react";

import { LevelCameraEffectSection } from "@/core/level/components/preview/toolbar/camera/LevelCameraEffectSection";
import { LevelSettingsResetButton } from "@/core/level/components/preview/toolbar/LevelSettingsResetButton";
import { useLevelFeatureOverride } from "@/core/level/components/preview/toolbar/use-level-feature-override";
import { useLevelRenderFeatures } from "@/core/level/components/preview/toolbar/use-level-render-features";
import { toLevelModeFeature } from "@/core/level/lib/features/level-feature-options";
import { LevelViewportService } from "@/core/level/services/level-viewport";
import { formatCameraFocusDistance } from "@/core/render/lib/features/image/render-camera-choices";
import { TRenderCameraSettings } from "@/core/render/lib/settings/choice/render-feature-settings";

/**
 * The camera this view's frame is shown through: its motion blur, its depth of field focused by a value or by a click
 * in the view, its vignette, its film grain and its chromatic aberration, each over the settings for the view alone.
 */
export function LevelCameraEffectsAction({
  "data-testid": dataTestId = "level-camera-effects-action",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const viewportService: LevelViewportService = useInjection(LevelViewportService);
  const { settings, features, drawn } = useLevelRenderFeatures();
  const { set, reset } = useLevelFeatureOverride("camera");

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
    >
      <LevelCameraEffectSection
        effect={"isMotionBlurred"}
        names={["shutter"]}
        camera={camera}
        features={drawn}
        onSet={set}
      />

      <LevelCameraEffectSection
        data-testid={"level-camera-depth-of-field"}
        effect={"isDefocused"}
        names={["fStop", "focusDistance"]}
        camera={camera}
        features={drawn}
        onSet={set}
      >
        <Button
          data-testid={"level-camera-focus-pick"}
          size={"small"}
          variant={isFocusPicking ? "contained" : "outlined"}
          startIcon={<CenterFocusStrongIcon />}
          onClick={() => viewportService.setFocusPicking(!isFocusPicking)}
        >
          {isFocusPicking ? "Click in the view to focus" : "Focus by a click"}
        </Button>
      </LevelCameraEffectSection>

      <LevelCameraEffectSection
        effect={"isVignetted"}
        names={["vignetteStrength"]}
        camera={camera}
        features={drawn}
        onSet={set}
      />

      <LevelCameraEffectSection
        effect={"isGrained"}
        names={["grainStrength", "grainSize"]}
        camera={camera}
        features={drawn}
        onSet={set}
      />

      <LevelCameraEffectSection
        effect={"isAberrated"}
        names={["aberrationStrength"]}
        camera={camera}
        features={drawn}
        onSet={set}
      />

      <LevelSettingsResetButton onReset={reset} />
    </EditorPopoverGroup>
  );
}
