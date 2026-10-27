import { default as HighQualityIcon } from "@mui/icons-material/HighQuality";
import { Button, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import {
  ERenderAntialiasing,
  ERenderLightModel,
  ERenderTonemapMode,
  RenderAntialiasing,
  RenderDlssAvailability,
  RenderDlssPreset,
  RenderRayReconstructionPreset,
  RenderScale,
} from "@xrf/ipc/types/xrf-renderer";
import { Nullable } from "@xrf/types";
import { ReactElement, useCallback } from "react";

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
import {
  describeRenderAntialiasing,
  isTemporalAntialiasing,
  toDrawnAntialiasing,
} from "@/core/render/lib/features/render-antialiasing-choices";
import {
  formatOutputScale,
  RENDER_OUTPUT_SCALE_LIMITS,
  RENDER_RESOLUTION_OPTIONS,
} from "@/core/render/lib/features/render-display-choices";
import {
  describeDlssPreset,
  describeDlssUnavailability,
  describeRayReconstructionPreset,
  describeRayReconstructionUnavailability,
  formatSharpening,
  RENDER_DLSS_PRESET_SHORT_OPTIONS,
  RENDER_RAY_RECONSTRUCTION_PRESET_OPTIONS,
  RENDER_SCALE_OPTIONS,
  RENDER_SHARPENING_LIMITS,
} from "@/core/render/lib/features/render-upscaling-choices";
import { ERenderResolution } from "@/core/render/lib/settings/render-resolution";
import { EditorPopoverAction } from "@/core/shell/editor/EditorPopoverAction";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { IChoiceFormRowOption } from "@/core/ui/form/ChoiceFormRow/choice-form-row-option";
import { BaseComponentProps } from "@/lib/dom/element-types";

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
  const { scale, sharpening, dlssPreset, rayReconstructionPreset: rayPreset } = drawn.upscaling;
  // Whether the scene holds linear light, the Enhanced tonemap or the physical light, which ray reconstruction needs.
  const isSceneLinear: boolean =
    drawn.tonemap === ERenderTonemapMode.ENHANCED || drawn.lightModel === ERenderLightModel.PHYSICAL;
  // Whether the frame is drawn smaller than the viewport, which sharpening applies to besides a temporal resolve.
  const isUpscaled: boolean = renderService.isUpscaled;
  // What the GPU can do: FSR smooths in place of DLSS, and DLSS upscales alone, where it lacks them.
  const dlss: RenderDlssAvailability = renderService.dlss;
  const rayReconstruction: RenderDlssAvailability = renderService.rayReconstruction;

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

        <RenderValueChoice
          label={"Upscaling quality"}
          options={RENDER_SCALE_OPTIONS}
          value={scale}
          isStacked
          onChange={(next: RenderScale) => setUpscaling({ scale: next })}
        />

        <CheckboxFormRow
          label={"Ray reconstruction"}
          description={
            isDlss
              ? (rayProblem ?? "Denoises and upscales in one network, over each surface's albedo and normal.")
              : "Comes with DLSS, under Mode."
          }
          isChecked={isDlss && isRayReconstructed}
          isDisabled={!isDlss || Boolean(rayProblem)}
          onChange={(next: boolean) => setUpscaling({ isRayReconstructed: next })}
        />

        <RenderValueChoice
          label={"DLSS model"}
          options={RENDER_DLSS_PRESET_SHORT_OPTIONS}
          value={dlssPreset}
          isDisabled={!isDlss || isRayReconstructed}
          onChange={(next: RenderDlssPreset) => setUpscaling({ dlssPreset: next })}
        />

        <RenderValueChoice
          label={"Ray reconstruction model"}
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

        <RenderValueSlider
          label={"Sharpening"}
          value={sharpening}
          {...RENDER_SHARPENING_LIMITS}
          format={formatSharpening}
          isDisabled={!isSharpenable}
          onChange={(next: number) => setUpscaling({ sharpening: next })}
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

        <Button size={"small"} onClick={reset}>
          Reset to the preset
        </Button>
      </div>
    </EditorPopoverAction>
  );
}
