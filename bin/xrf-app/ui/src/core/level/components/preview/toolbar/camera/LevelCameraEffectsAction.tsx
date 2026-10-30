import { useInjection } from "@wirestate/react";
import { EditorPopoverGroup, EditorPopoverGroupSection } from "@xrf/ui/editor/popover-group";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { CameraOutlinedIcon } from "@xrf/ui/icons/camera-outlined";
import { CenterFocusStrongIcon } from "@xrf/ui/icons/center-focus-strong";
import { Button } from "@xrf/ui/material/button";
import { ReactElement } from "react";

import { useLevelFeatureOverride } from "@/core/level/components/preview/toolbar/use-level-feature-override";
import { useLevelRenderFeatures } from "@/core/level/components/preview/toolbar/use-level-render-features";
import { toLevelModeFeature } from "@/core/level/lib/features/level-feature-options";
import { LevelViewportService } from "@/core/level/services/level-viewport";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import {
  formatCameraFocusDistance,
  formatCameraFStop,
  formatCameraGrainSize,
  formatCameraShutter,
  formatCameraStrength,
  RENDER_CAMERA_LIMITS,
} from "@/core/render/lib/features/render-camera-choices";
import { TRenderCameraSettings } from "@/core/render/lib/settings/render-feature-settings";

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
  const { settings, features } = useLevelRenderFeatures();
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

  return (
    <EditorPopoverGroup
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Camera effects"}
      description={effects.length ? `Camera effects: ${effects.join(", ")}` : "No camera effects"}
      icon={<CameraOutlinedIcon />}
      isActive={effects.length > 0}
    >
      <EditorPopoverGroupSection
        label={"Motion blur"}
        description={"Blurs what moves across the frame, and the frame as the camera turns"}
        isOn={camera.isMotionBlurred}
        onToggle={() => set({ isMotionBlurred: !camera.isMotionBlurred })}
      >
        <RenderValueSlider
          label={"Shutter"}
          value={camera.shutter}
          {...RENDER_CAMERA_LIMITS.shutter}
          isDisabled={!camera.isMotionBlurred}
          format={formatCameraShutter}
          onChange={(shutter: number) => set({ shutter })}
        />
      </EditorPopoverGroupSection>

      <EditorPopoverGroupSection
        data-testid={"level-camera-depth-of-field"}
        label={"Depth of field"}
        description={"Blurs what stands nearer or farther than the focus"}
        isOn={camera.isDefocused}
        onToggle={() => set({ isDefocused: !camera.isDefocused })}
      >
        <RenderValueSlider
          label={"Aperture"}
          value={camera.fStop}
          {...RENDER_CAMERA_LIMITS.fStop}
          isDisabled={!camera.isDefocused}
          format={formatCameraFStop}
          onChange={(fStop: number) => set({ fStop })}
        />

        <RenderValueSlider
          label={"Focus"}
          value={camera.focusDistance}
          {...RENDER_CAMERA_LIMITS.focusDistance}
          isDisabled={!camera.isDefocused}
          format={formatCameraFocusDistance}
          onChange={(focusDistance: number) => set({ focusDistance })}
        />

        <Button
          data-testid={"level-camera-focus-pick"}
          size={"small"}
          variant={isFocusPicking ? "contained" : "outlined"}
          startIcon={<CenterFocusStrongIcon />}
          onClick={() => viewportService.setFocusPicking(!isFocusPicking)}
        >
          {isFocusPicking ? "Click in the view to focus" : "Focus by a click"}
        </Button>
      </EditorPopoverGroupSection>

      <EditorPopoverGroupSection
        label={"Vignette"}
        description={"Dims the frame's corners as a lens does"}
        isOn={camera.isVignetted}
        onToggle={() => set({ isVignetted: !camera.isVignetted })}
      >
        <RenderValueSlider
          label={"Strength"}
          value={camera.vignetteStrength}
          {...RENDER_CAMERA_LIMITS.vignetteStrength}
          isDisabled={!camera.isVignetted}
          format={formatCameraStrength}
          onChange={(vignetteStrength: number) => set({ vignetteStrength })}
        />
      </EditorPopoverGroupSection>

      <EditorPopoverGroupSection
        label={"Film grain"}
        description={"A fine grain over the frame, different every frame"}
        isOn={camera.isGrained}
        onToggle={() => set({ isGrained: !camera.isGrained })}
      >
        <RenderValueSlider
          label={"Strength"}
          value={camera.grainStrength}
          {...RENDER_CAMERA_LIMITS.grainStrength}
          isDisabled={!camera.isGrained}
          format={formatCameraStrength}
          onChange={(grainStrength: number) => set({ grainStrength })}
        />

        <RenderValueSlider
          label={"Size"}
          value={camera.grainSize}
          {...RENDER_CAMERA_LIMITS.grainSize}
          isDisabled={!camera.isGrained}
          format={formatCameraGrainSize}
          onChange={(grainSize: number) => set({ grainSize })}
        />
      </EditorPopoverGroupSection>

      <EditorPopoverGroupSection
        label={"Chromatic aberration"}
        description={"Splits red and blue apart towards the frame's edges"}
        isOn={camera.isAberrated}
        onToggle={() => set({ isAberrated: !camera.isAberrated })}
      >
        <RenderValueSlider
          label={"Strength"}
          value={camera.aberrationStrength}
          {...RENDER_CAMERA_LIMITS.aberrationStrength}
          isDisabled={!camera.isAberrated}
          format={formatCameraStrength}
          onChange={(aberrationStrength: number) => set({ aberrationStrength })}
        />
      </EditorPopoverGroupSection>

      <Button size={"small"} onClick={reset}>
        Reset to the preset
      </Button>
    </EditorPopoverGroup>
  );
}
