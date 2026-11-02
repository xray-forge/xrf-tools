import { useInjection } from "@wirestate/react";
import {
  ERenderAntialiasing,
  RenderAntialiasing,
  RenderDlssAvailability,
  RenderDlssPreset,
  RenderRayReconstructionPreset,
} from "@xrf/ipc/types/xrf-renderer";
import { Nullable } from "@xrf/types";
import { EditorPopoverAction } from "@xrf/ui/editor/popover-action";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { CheckboxFormRow } from "@xrf/ui/form/checkbox-row";
import { IChoiceFormRowOption } from "@xrf/ui/form/choice-row";
import { HighQualityIcon } from "@xrf/ui/icons/high-quality";
import { Typography } from "@xrf/ui/material/typography";
import { ReactElement, useCallback } from "react";

import { LevelSettingsResetButton } from "@/core/level/components/preview/toolbar/LevelSettingsResetButton";
import { useLevelFeatureOverride } from "@/core/level/components/preview/toolbar/use-level-feature-override";
import { useLevelRenderFeatures } from "@/core/level/components/preview/toolbar/use-level-render-features";
import { useLevelViewToggle } from "@/core/level/components/preview/toolbar/use-level-view-toggle";
import {
  describeLevelFeatureToggle,
  ILevelFeatureOptions,
  LEVEL_ANTIALIASING_MODES,
} from "@/core/level/lib/features/level-feature-options";
import { LevelRenderService } from "@/core/level/services/level-render";
import { LevelViewService } from "@/core/level/services/level-view";
import { RenderValueChoice } from "@/core/render/components/controls/RenderValueChoice";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import { RenderSettingEntry } from "@/core/render/components/settings/RenderSettingEntry";
import {
  describeRenderAntialiasing,
  isTemporalAntialiasing,
  toDrawnAntialiasing,
} from "@/core/render/lib/features/image/render-antialiasing-choices";
import {
  formatOutputScale,
  RENDER_OUTPUT_SCALE_LIMITS,
  RENDER_RESOLUTION_OPTIONS,
} from "@/core/render/lib/features/image/render-display-choices";
import {
  describeDlssPreset,
  describeDlssUnavailability,
  describeRayReconstructionPreset,
  describeRayReconstructionUnavailability,
  isRenderSceneLinear,
  RENDER_DLSS_PRESET_SHORT_OPTIONS,
  RENDER_RAY_RECONSTRUCTION_PRESET_OPTIONS,
  RENDER_UPSCALING_SETTINGS,
} from "@/core/render/lib/features/image/render-upscaling-choices";
import { ERenderResolution } from "@/core/render/lib/settings/device/render-resolution";

const MODE_OPTIONS: ReadonlyArray<IChoiceFormRowOption<RenderAntialiasing>> = LEVEL_ANTIALIASING_MODES.map(
  (value: RenderAntialiasing) => ({ label: describeRenderAntialiasing(value), value })
);

/** What the popover sets for the view, which its reset leaves to the settings again. */
const IMAGE_QUALITY_FEATURES: ReadonlyArray<keyof ILevelFeatureOptions> = ["antialiasing", "upscaling", "output"];

/** The modes offered where the GPU has no DLSS: DLSS greyed out, FSR standing in for it. */
const MODE_OPTIONS_WITHOUT_DLSS: ReadonlyArray<IChoiceFormRowOption<RenderAntialiasing>> = MODE_OPTIONS.map(
  (option: IChoiceFormRowOption<RenderAntialiasing>) =>
    option.value === ERenderAntialiasing.DLSS ? { ...option, isDisabled: true } : option
);

/**
 * How this view's frame is made into the image it shows: whether its edges are smoothed and by which pass, whether DLSS
 * reconstructs rays as it upscales, how much smaller it is drawn, and what it is resolved at.
 */
export function LevelImageQualityAction({
  "data-testid": dataTestId = "level-image-quality-action",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const viewService: LevelViewService = useInjection(LevelViewService);
  const renderService: LevelRenderService = useInjection(LevelRenderService);
  const { settings, features, drawn, output } = useLevelRenderFeatures();
  const { isOn, toggle } = useLevelViewToggle("isAntialiased");
  const { set: setUpscaling } = useLevelFeatureOverride("upscaling");
  const { set: setOutput } = useLevelFeatureOverride("output");
  const { dlssPreset, rayReconstructionPreset: rayPreset } = drawn.upscaling;
  // Whether the scene holds linear light, which ray reconstruction needs.
  const isSceneLinear: boolean = isRenderSceneLinear(drawn);
  // Whether the frame is drawn smaller than the viewport, which sharpening applies to besides a temporal resolve.
  const isUpscaled: boolean = renderService.drawing.isUpscaled;
  // What the GPU can do: FSR smooths in place of DLSS, and DLSS upscales alone, where it lacks them.
  const dlss: RenderDlssAvailability = renderService.drawing.dlss;
  const rayReconstruction: RenderDlssAvailability = renderService.drawing.rayReconstruction;

  // The settings' mode is the view's to narrow to none alone: it cannot smooth what the settings do not.
  const isAvailable: boolean = settings.antialiasing !== ERenderAntialiasing.NONE;
  const dlssProblem: Nullable<string> = describeDlssUnavailability(dlss);
  const mode: RenderAntialiasing = toDrawnAntialiasing(drawn.antialiasing, dlss);
  const rayProblem: Nullable<string> = describeRayReconstructionUnavailability(rayReconstruction, isSceneLinear);
  const isRayReconstructed: boolean = !rayProblem && drawn.upscaling.isRayReconstructed;
  const isSharpenable: boolean = isUpscaled || (isOn && isTemporalAntialiasing(mode));
  const isDlss: boolean = isAvailable && mode === ERenderAntialiasing.DLSS;

  const description: string = describeLevelFeatureToggle({
    isAvailable,
    isOn,
    label: "Antialiasing",
    off: "Antialiasing off, every edge as drawn",
    on: `Edges smoothed by ${describeRenderAntialiasing(mode)}`,
  });

  const reset = useCallback(() => viewService.resetFeatures(IMAGE_QUALITY_FEATURES), [viewService]);

  return (
    <EditorPopoverAction
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Image quality"}
      description={description}
      icon={<HighQualityIcon />}
      isActive={isOn && isAvailable}
    >
      <div className={"flex w-80 flex-col gap-2 px-4 py-2"}>
        <Typography className={"text-text-secondary"} variant={"overline"}>
          Image quality
        </Typography>

        <CheckboxFormRow
          label={"Antialiasing"}
          description={isAvailable ? undefined : description}
          isChecked={isOn && isAvailable}
          isDisabled={!isAvailable}
          onChange={toggle}
        />

        <RenderValueChoice
          label={"Mode"}
          options={dlssProblem ? MODE_OPTIONS_WITHOUT_DLSS : MODE_OPTIONS}
          value={mode}
          isDisabled={!isAvailable}
          onChange={(antialiasing: RenderAntialiasing) => viewService.setFeatures({ ...features, antialiasing })}
        />

        {isAvailable && dlssProblem ? (
          <Typography className={"text-text-secondary"} variant={"caption"}>
            {dlssProblem}
          </Typography>
        ) : null}

        <RenderSettingEntry
          settings={RENDER_UPSCALING_SETTINGS}
          name={"scale"}
          group={drawn.upscaling}
          features={drawn}
          onSet={setUpscaling}
        />

        <CheckboxFormRow
          label={RENDER_UPSCALING_SETTINGS.isRayReconstructed.label}
          description={
            isDlss
              ? (rayProblem ?? (RENDER_UPSCALING_SETTINGS.isRayReconstructed.description as string))
              : "Comes with DLSS, under Mode."
          }
          isChecked={isDlss && isRayReconstructed}
          isDisabled={!isDlss || Boolean(rayProblem)}
          onChange={(next: boolean) => setUpscaling({ isRayReconstructed: next })}
        />

        <RenderValueChoice
          label={RENDER_UPSCALING_SETTINGS.dlssPreset.label}
          options={RENDER_DLSS_PRESET_SHORT_OPTIONS}
          value={dlssPreset}
          isDisabled={!isDlss || isRayReconstructed}
          onChange={(next: RenderDlssPreset) => setUpscaling({ dlssPreset: next })}
        />

        <RenderValueChoice
          label={RENDER_UPSCALING_SETTINGS.rayReconstructionPreset.label}
          options={RENDER_RAY_RECONSTRUCTION_PRESET_OPTIONS}
          value={rayPreset}
          isDisabled={!isDlss || !isRayReconstructed}
          onChange={(next: RenderRayReconstructionPreset) => setUpscaling({ rayReconstructionPreset: next })}
        />

        {isDlss ? (
          <Typography className={"text-text-secondary"} variant={"caption"}>
            {isRayReconstructed ? describeRayReconstructionPreset(rayPreset) : describeDlssPreset(dlssPreset)}
          </Typography>
        ) : null}

        <RenderSettingEntry
          settings={RENDER_UPSCALING_SETTINGS}
          name={"sharpening"}
          group={drawn.upscaling}
          features={drawn}
          isDisabled={!isSharpenable}
          onSet={setUpscaling}
        />

        {isSharpenable ? null : (
          <Typography className={"text-text-secondary"} variant={"caption"}>
            Sharpens only a frame drawn smaller than the viewport or resolved temporally.
          </Typography>
        )}

        <RenderValueChoice
          label={"Output resolution"}
          options={RENDER_RESOLUTION_OPTIONS}
          value={output.resolution}
          onChange={(next: ERenderResolution) => setOutput({ resolution: next })}
        />

        <RenderValueSlider
          label={"Output scale"}
          value={output.scale}
          {...RENDER_OUTPUT_SCALE_LIMITS}
          isDisabled={output.resolution !== ERenderResolution.AUTO}
          format={formatOutputScale}
          onChange={(next: number) => setOutput({ scale: next })}
        />

        <LevelSettingsResetButton onReset={reset} />
      </div>
    </EditorPopoverAction>
  );
}
