import { default as HighQualityIcon } from "@mui/icons-material/HighQuality";
import { Button, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { ReactElement } from "react";

import {
  ERenderAntialiasing,
  ERenderTonemapMode,
  RenderAntialiasing,
  RenderDlssAvailability,
  RenderDlssPreset,
  RenderRayReconstructionPreset,
  RenderScale,
} from "@/core/ipc/types/xrf-renderer";
import { useLevelFeatureOverride } from "@/core/level/components/preview/toolbar/use-level-feature-override";
import { useLevelRenderFeatures } from "@/core/level/components/preview/toolbar/use-level-render-features";
import { useLevelViewToggle } from "@/core/level/components/preview/toolbar/use-level-view-toggle";
import { describeLevelFeatureToggle, LEVEL_ANTIALIASING_MODES } from "@/core/level/lib/features/level-feature-options";
import { LevelRenderService } from "@/core/level/services/level-render.service";
import { LevelViewService } from "@/core/level/services/level-view.service";
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
import { RenderSettingsService } from "@/core/render/services/render-settings";
import { EditorPopoverAction } from "@/core/shell/editor/EditorPopoverAction";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { IChoiceFormRowOption } from "@/core/ui/form/ChoiceFormRow/choice-form-row-option";
import { BaseComponentProps } from "@/lib/dom/element-types";

const MODE_OPTIONS: ReadonlyArray<IChoiceFormRowOption<RenderAntialiasing>> = LEVEL_ANTIALIASING_MODES.map(
  (value: RenderAntialiasing) => ({ label: describeRenderAntialiasing(value), value })
);

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
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);
  const { settings, features } = useLevelRenderFeatures();
  const { isOn, toggle } = useLevelViewToggle("isAntialiased");
  const { set: setUpscaling } = useLevelFeatureOverride("upscaling");
  const { set: setOutput } = useLevelFeatureOverride("output");
  // What the settings draw every viewport with, which this view may set over, the mode only narrowed to none.
  const settingsMode: RenderAntialiasing = settings.antialiasing;
  const settingsScale: RenderScale = settings.upscaling.scale;
  const settingsSharpening: number = settings.upscaling.sharpening;
  const settingsDlssPreset: RenderDlssPreset = settings.upscaling.dlssPreset;
  const settingsRayReconstruction: boolean = settings.upscaling.isRayReconstructed;
  const settingsRayPreset: RenderRayReconstructionPreset = settings.upscaling.rayReconstructionPreset;
  const settingsResolution: ERenderResolution = renderSettingsService.renderResolution;
  const settingsOutputScale: number = renderSettingsService.renderOutputScale;
  // Whether the scene holds linear light, the Enhanced tonemap, which ray reconstruction needs.
  const isSceneLinear: boolean = settings.tonemap === ERenderTonemapMode.ENHANCED;
  // Whether the frame is drawn smaller than the viewport, which sharpening applies to besides a temporal resolve.
  const isUpscaled: boolean = renderService.isUpscaled;
  // What the GPU can do: FSR smooths in place of DLSS, and DLSS upscales alone, where it lacks them.
  const dlss: RenderDlssAvailability = renderService.dlss;
  const rayReconstruction: RenderDlssAvailability = renderService.rayReconstruction;

  const isAvailable: boolean = settingsMode !== ERenderAntialiasing.NONE;
  const asked: RenderAntialiasing = features.antialiasing ?? settingsMode;
  const dlssProblem: Nullable<string> = describeDlssUnavailability(dlss);
  const mode: RenderAntialiasing = toDrawnAntialiasing(asked, dlss);
  const scale: RenderScale = features.upscaling.scale ?? settingsScale;
  const sharpening: number = features.upscaling.sharpening ?? settingsSharpening;
  const dlssPreset: RenderDlssPreset = features.upscaling.dlssPreset ?? settingsDlssPreset;
  const rayPreset: RenderRayReconstructionPreset = features.upscaling.rayReconstructionPreset ?? settingsRayPreset;
  const rayProblem: Nullable<string> = describeRayReconstructionUnavailability(rayReconstruction, isSceneLinear);
  const isRayReconstructed: boolean =
    !rayProblem && (features.upscaling.isRayReconstructed ?? settingsRayReconstruction);
  const resolution: ERenderResolution = features.output.resolution ?? settingsResolution;
  const outputScale: number = features.output.scale ?? settingsOutputScale;
  const isSharpenable: boolean = isUpscaled || (isOn && isTemporalAntialiasing(mode));

  const description: string = describeLevelFeatureToggle({
    isAvailable,
    isOn,
    label: "Antialiasing",
    off: "Antialiasing off, every edge as drawn",
    on: `Edges smoothed by ${describeRenderAntialiasing(mode)}`,
  });

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

        {isAvailable ? (
          <RenderValueChoice
            label={"Mode"}
            options={dlssProblem ? MODE_OPTIONS_WITHOUT_DLSS : MODE_OPTIONS}
            value={mode}
            onChange={(antialiasing: RenderAntialiasing) => viewService.setFeatures({ ...features, antialiasing })}
          />
        ) : null}

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

        {isAvailable && mode === ERenderAntialiasing.DLSS ? (
          <>
            <CheckboxFormRow
              label={"Ray reconstruction"}
              description={rayProblem ?? "Denoises and upscales in one network, over each surface's albedo and normal."}
              isChecked={isRayReconstructed}
              isDisabled={Boolean(rayProblem)}
              onChange={(next: boolean) => setUpscaling({ isRayReconstructed: next })}
            />

            {isRayReconstructed ? (
              <>
                <RenderValueChoice
                  label={"Ray reconstruction model"}
                  options={RENDER_RAY_RECONSTRUCTION_PRESET_OPTIONS}
                  value={rayPreset}
                  onChange={(next: RenderRayReconstructionPreset) => setUpscaling({ rayReconstructionPreset: next })}
                />

                <Typography className={"text-text-secondary"} variant={"caption"}>
                  {describeRayReconstructionPreset(rayPreset)}
                </Typography>
              </>
            ) : (
              <>
                <RenderValueChoice
                  label={"DLSS model"}
                  options={RENDER_DLSS_PRESET_SHORT_OPTIONS}
                  value={dlssPreset}
                  onChange={(next: RenderDlssPreset) => setUpscaling({ dlssPreset: next })}
                />

                <Typography className={"text-text-secondary"} variant={"caption"}>
                  {describeDlssPreset(dlssPreset)}
                </Typography>
              </>
            )}
          </>
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
          value={resolution}
          onChange={(next: ERenderResolution) => setOutput({ resolution: next })}
        />

        {resolution === ERenderResolution.AUTO ? (
          <RenderValueSlider
            label={"Output scale"}
            value={outputScale}
            {...RENDER_OUTPUT_SCALE_LIMITS}
            format={formatOutputScale}
            onChange={(next: number) => setOutput({ scale: next })}
          />
        ) : null}

        <Button
          size={"small"}
          onClick={() =>
            viewService.setFeatures({
              ...features,
              antialiasing: null,
              upscaling: {},
              output: {},
            })
          }
        >
          Reset to the preset
        </Button>
      </div>
    </EditorPopoverAction>
  );
}
