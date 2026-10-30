import { useInjection } from "@wirestate/react";
import { CheckboxFormRow } from "@xrf/ui/form/checkbox-row";
import { SliderFormRow } from "@xrf/ui/form/slider-row";
import { DetailSection } from "@xrf/ui/layout/detail-section";
import { ReactElement } from "react";

import {
  formatCameraFocusDistance,
  formatCameraFStop,
  formatCameraGrainSize,
  formatCameraMotionLimit,
  formatCameraSensorHeight,
  formatCameraShutter,
  formatCameraStrength,
  RENDER_CAMERA_LIMITS,
} from "@/core/render/lib/features/render-camera-choices";
import { TRenderCameraSettings } from "@/core/render/lib/settings/render-feature-settings";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";

import { useRendererOverride } from "./use-renderer-override";

/**
 * The camera the frame is shown through: motion blur, depth of field, vignette, film grain and chromatic aberration,
 * each off until turned on, and what shapes it.
 */
export function SettingsRendererCamera(): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);
  const onSet = useRendererOverride("camera");
  const camera: TRenderCameraSettings = renderSettingsService.rendererFeatures.camera;

  return (
    <DetailSection
      title={"Camera"}
      description={"A physical camera's effects over the finished frame, each off until turned on."}
    >
      <div className={"mt-4 flex flex-col gap-6"}>
        <CheckboxFormRow
          label={"Motion blur"}
          description={"Blurs what moves across the frame, and the frame as the camera turns, along its motion."}
          isChecked={camera.isMotionBlurred}
          onChange={(isMotionBlurred: boolean) => onSet({ isMotionBlurred })}
        />

        <SliderFormRow
          label={"Shutter"}
          description={"How long the shutter stays open each frame, as a film camera's angle: 180° is film's look."}
          value={camera.shutter}
          {...RENDER_CAMERA_LIMITS.shutter}
          isDisabled={!camera.isMotionBlurred}
          format={formatCameraShutter}
          onChange={(shutter: number) => onSet({ shutter })}
        />

        <SliderFormRow
          label={"Longest blur"}
          description={"How far a blur reaches at most, so a fast turn smears no further."}
          value={camera.motionLimit}
          {...RENDER_CAMERA_LIMITS.motionLimit}
          isDisabled={!camera.isMotionBlurred}
          format={formatCameraMotionLimit}
          onChange={(motionLimit: number) => onSet({ motionLimit })}
        />

        <CheckboxFormRow
          label={"Depth of field"}
          description={
            "Blurs what stands nearer or farther than the focus, as a lens of the view's field of view does. A pick in " +
            "the viewer sets the focus."
          }
          isChecked={camera.isDefocused}
          onChange={(isDefocused: boolean) => onSet({ isDefocused })}
        />

        <SliderFormRow
          label={"Aperture"}
          description={"The lens's f-number: lower opens it wider and keeps less in focus."}
          value={camera.fStop}
          {...RENDER_CAMERA_LIMITS.fStop}
          isDisabled={!camera.isDefocused}
          format={formatCameraFStop}
          onChange={(fStop: number) => onSet({ fStop })}
        />

        <SliderFormRow
          label={"Focus"}
          description={"How far from the camera the lens is focused."}
          value={camera.focusDistance}
          {...RENDER_CAMERA_LIMITS.focusDistance}
          isDisabled={!camera.isDefocused}
          format={formatCameraFocusDistance}
          onChange={(focusDistance: number) => onSet({ focusDistance })}
        />

        <SliderFormRow
          label={"Sensor"}
          description={"The sensor's height: larger blurs more at the same view and aperture. 24 mm is full frame."}
          value={camera.sensorHeight}
          {...RENDER_CAMERA_LIMITS.sensorHeight}
          isDisabled={!camera.isDefocused}
          format={formatCameraSensorHeight}
          onChange={(sensorHeight: number) => onSet({ sensorHeight })}
        />

        <CheckboxFormRow
          label={"Vignette"}
          description={"Dims the frame's corners, as light reaching the film at an angle falls off through a lens."}
          isChecked={camera.isVignetted}
          onChange={(isVignetted: boolean) => onSet({ isVignetted })}
        />

        <SliderFormRow
          label={"Vignette strength"}
          description={"How much of the lens's falloff the corners show."}
          value={camera.vignetteStrength}
          {...RENDER_CAMERA_LIMITS.vignetteStrength}
          isDisabled={!camera.isVignetted}
          format={formatCameraStrength}
          onChange={(vignetteStrength: number) => onSet({ vignetteStrength })}
        />

        <CheckboxFormRow
          label={"Film grain"}
          description={"Lays a fine grain over the frame, different every frame, strongest in the mid tones."}
          isChecked={camera.isGrained}
          onChange={(isGrained: boolean) => onSet({ isGrained })}
        />

        <SliderFormRow
          label={"Grain strength"}
          description={"How strongly the grain shows."}
          value={camera.grainStrength}
          {...RENDER_CAMERA_LIMITS.grainStrength}
          isDisabled={!camera.isGrained}
          format={formatCameraStrength}
          onChange={(grainStrength: number) => onSet({ grainStrength })}
        />

        <SliderFormRow
          label={"Grain size"}
          description={"How many pixels across a grain is."}
          value={camera.grainSize}
          {...RENDER_CAMERA_LIMITS.grainSize}
          isDisabled={!camera.isGrained}
          format={formatCameraGrainSize}
          onChange={(grainSize: number) => onSet({ grainSize })}
        />

        <CheckboxFormRow
          label={"Chromatic aberration"}
          description={"Splits red and blue apart towards the frame's edges, as a lens's glass bends them unequally."}
          isChecked={camera.isAberrated}
          onChange={(isAberrated: boolean) => onSet({ isAberrated })}
        />

        <SliderFormRow
          label={"Aberration strength"}
          description={"How far apart the colours split at the corners."}
          value={camera.aberrationStrength}
          {...RENDER_CAMERA_LIMITS.aberrationStrength}
          isDisabled={!camera.isAberrated}
          format={formatCameraStrength}
          onChange={(aberrationStrength: number) => onSet({ aberrationStrength })}
        />
      </div>
    </DetailSection>
  );
}
