import { default as HighQualityIcon } from "@mui/icons-material/HighQuality";
import { Button, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { ReactElement, useEffect } from "react";

import {
  ERenderAntialiasing,
  ERenderTonemapMode,
  RenderAntialiasing,
  RenderBackendAvailability,
  RenderDlssAvailability,
  RenderDlssPreset,
  RenderScale,
  RenderTextureFiltering,
} from "@/core/ipc/types/xrf-renderer";
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
  RENDER_TEXTURE_FILTERING_SHORT_OPTIONS,
} from "@/core/render/lib/features/render-antialiasing-choices";
import {
  formatOutputScale,
  RENDER_OUTPUT_SCALE_LIMITS,
  RENDER_RESOLUTION_OPTIONS,
} from "@/core/render/lib/features/render-display-choices";
import {
  describeDlssPreset,
  describeDlssUnavailability,
  describeRayReconstructionUnavailability,
  formatSharpening,
  RENDER_DLSS_PRESET_SHORT_OPTIONS,
  RENDER_SCALE_OPTIONS,
  RENDER_SHARPENING_LIMITS,
} from "@/core/render/lib/features/render-upscaling-choices";
import {
  describeRenderBackend,
  isRenderBackendAvailable,
  RENDER_BACKEND_AUTO,
  RENDER_BACKEND_CHOICES,
  TRenderBackendChoice,
} from "@/core/render/lib/settings/render-backend-choice";
import { ERenderResolution } from "@/core/render/lib/settings/render-resolution";
import { RenderSettingsService } from "@/core/render/services/render-settings";
import { EditorPopoverToggle } from "@/core/shell/editor/EditorPopoverToggle";
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
 * @param availability - What each graphics API can do here, none until known.
 * @returns The graphics APIs offered: automatic, then each by name, greyed out where this machine cannot draw with it.
 */
function toBackendOptions(
  availability: Nullable<ReadonlyArray<RenderBackendAvailability>>
): ReadonlyArray<IChoiceFormRowOption<TRenderBackendChoice>> {
  return RENDER_BACKEND_CHOICES.map((choice: TRenderBackendChoice) =>
    choice === RENDER_BACKEND_AUTO
      ? { label: "Auto", value: choice }
      : {
          isDisabled: availability !== null && !isRenderBackendAvailable(choice, availability),
          label: describeRenderBackend(choice),
          value: choice,
        }
  );
}

/**
 * How this view's frame is made into the image it shows: whether its edges are smoothed and by which pass, whether DLSS
 * reconstructs rays as it upscales, how much smaller it is drawn, what it is resolved at, and how its textures are
 * sampled.
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
  // What the settings draw every viewport with, which this view may set over, the mode only narrowed to none.
  const settingsMode: RenderAntialiasing = settings.antialiasing;
  const settingsTextureFiltering: RenderTextureFiltering = settings.textureFiltering;
  const settingsScale: RenderScale = settings.upscaling.scale;
  const settingsSharpening: number = settings.upscaling.sharpening;
  const settingsDlssPreset: RenderDlssPreset = settings.upscaling.dlssPreset;
  const settingsRayReconstruction: boolean = settings.upscaling.isRayReconstructed;
  const settingsResolution: ERenderResolution = renderSettingsService.renderResolution;
  const settingsOutputScale: number = renderSettingsService.renderOutputScale;
  // Whether the scene holds linear light, the Enhanced tonemap, which ray reconstruction needs.
  const isSceneLinear: boolean = settings.tonemap === ERenderTonemapMode.ENHANCED;
  // The graphics API every viewport draws with, set here for all of them, which restarts the renderer.
  const backend: TRenderBackendChoice = renderSettingsService.renderBackend;
  const backends: Nullable<ReadonlyArray<RenderBackendAvailability>> = renderSettingsService.renderBackends;
  // Whether the frame is drawn smaller than the viewport, which sharpening applies to besides a temporal resolve.
  const isUpscaled: boolean = renderService.isUpscaled;
  // What the GPU can do: FSR smooths in place of DLSS, and DLSS upscales alone, where it lacks them.
  const dlss: RenderDlssAvailability = renderService.dlss;
  const rayReconstruction: RenderDlssAvailability = renderService.rayReconstruction;

  const isAvailable: boolean = settingsMode !== ERenderAntialiasing.NONE;
  const asked: RenderAntialiasing = features.antialiasing ?? settingsMode;
  const dlssProblem: Nullable<string> = describeDlssUnavailability(dlss);
  const mode: RenderAntialiasing = asked === ERenderAntialiasing.DLSS && dlssProblem ? ERenderAntialiasing.FSR : asked;
  const textureFiltering: RenderTextureFiltering = features.textureFiltering ?? settingsTextureFiltering;
  const scale: RenderScale = features.upscaling.scale ?? settingsScale;
  const sharpening: number = features.upscaling.sharpening ?? settingsSharpening;
  const dlssPreset: RenderDlssPreset = features.upscaling.dlssPreset ?? settingsDlssPreset;
  const rayProblem: Nullable<string> = describeRayReconstructionUnavailability(rayReconstruction, isSceneLinear);
  const isRayReconstructed: boolean =
    !rayProblem && (features.upscaling.isRayReconstructed ?? settingsRayReconstruction);
  const resolution: ERenderResolution = features.output.resolution ?? settingsResolution;
  const outputScale: number = features.output.scale ?? settingsOutputScale;
  const isSharpenable: boolean = isUpscaled || (isOn && isTemporalAntialiasing(mode));

  // What each graphics API can do here, which the choices are greyed out by; asked once, while nothing has.
  useEffect(() => {
    if (renderSettingsService.renderBackends === null) {
      void renderSettingsService.refreshRenderBackends();
    }
  }, [renderSettingsService]);

  return (
    <EditorPopoverToggle
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Image quality"}
      description={describeLevelFeatureToggle({
        isAvailable,
        isOn,
        label: "Image quality",
        off: "Antialiasing off, every edge as drawn",
        on: `Edges smoothed by ${describeRenderAntialiasing(mode)}`,
      })}
      icon={<HighQualityIcon />}
      isOn={isOn && isAvailable}
      isDisabled={!isAvailable}
      isWide
      header={
        <>
          <RenderValueChoice
            label={"Graphics API"}
            options={toBackendOptions(backends)}
            value={backend}
            onChange={renderSettingsService.setRenderBackend}
          />

          <Typography className={"text-text-secondary"} variant={"caption"}>
            Every viewport draws with it; another one restarts the renderer.
          </Typography>
        </>
      }
      toggleLabel={"Antialiasing"}
      onToggle={toggle}
    >
      <RenderValueChoice
        label={"Mode"}
        options={dlssProblem ? MODE_OPTIONS_WITHOUT_DLSS : MODE_OPTIONS}
        value={mode}
        onChange={(antialiasing: RenderAntialiasing) => viewService.setFeatures({ ...features, antialiasing })}
      />

      {dlssProblem ? (
        <Typography className={"text-text-secondary"} variant={"caption"}>
          {dlssProblem}
        </Typography>
      ) : null}

      <RenderValueChoice
        label={"Upscaling quality"}
        options={RENDER_SCALE_OPTIONS}
        value={scale}
        isStacked
        onChange={(next: RenderScale) =>
          viewService.setFeatures({ ...features, upscaling: { ...features.upscaling, scale: next } })
        }
      />

      {mode === ERenderAntialiasing.DLSS ? (
        <>
          <CheckboxFormRow
            label={"Ray reconstruction"}
            description={rayProblem ?? "Denoises and upscales in one network, over each surface's albedo and normal."}
            isChecked={isRayReconstructed}
            isDisabled={Boolean(rayProblem)}
            onChange={(next: boolean) =>
              viewService.setFeatures({ ...features, upscaling: { ...features.upscaling, isRayReconstructed: next } })
            }
          />

          {isRayReconstructed ? null : (
            <>
              <RenderValueChoice
                label={"DLSS model"}
                options={RENDER_DLSS_PRESET_SHORT_OPTIONS}
                value={dlssPreset}
                onChange={(next: RenderDlssPreset) =>
                  viewService.setFeatures({ ...features, upscaling: { ...features.upscaling, dlssPreset: next } })
                }
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
        onChange={(next: number) =>
          viewService.setFeatures({ ...features, upscaling: { ...features.upscaling, sharpening: next } })
        }
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
        onChange={(next: ERenderResolution) =>
          viewService.setFeatures({ ...features, output: { ...features.output, resolution: next } })
        }
      />

      {resolution === ERenderResolution.AUTO ? (
        <RenderValueSlider
          label={"Output scale"}
          value={outputScale}
          {...RENDER_OUTPUT_SCALE_LIMITS}
          format={formatOutputScale}
          onChange={(next: number) =>
            viewService.setFeatures({ ...features, output: { ...features.output, scale: next } })
          }
        />
      ) : null}

      <RenderValueChoice
        label={"Texture filtering"}
        options={RENDER_TEXTURE_FILTERING_SHORT_OPTIONS}
        value={textureFiltering}
        onChange={(filtering: RenderTextureFiltering) =>
          viewService.setFeatures({ ...features, textureFiltering: filtering })
        }
      />

      <Button
        size={"small"}
        onClick={() =>
          viewService.setFeatures({
            ...features,
            antialiasing: null,
            textureFiltering: null,
            upscaling: {},
            output: {},
          })
        }
      >
        Back to the settings
      </Button>
    </EditorPopoverToggle>
  );
}
