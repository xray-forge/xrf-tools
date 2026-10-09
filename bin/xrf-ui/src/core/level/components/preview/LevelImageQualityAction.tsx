import { default as HighQualityIcon } from "@mui/icons-material/HighQuality";
import { Button, Typography } from "@mui/material";
import { Nullable } from "@xrf/types";
import { ReactElement } from "react";

import {
  ERenderAntialiasing,
  RenderAntialiasing,
  RenderBackendAvailability,
  RenderDlssAvailability,
  RenderDlssPreset,
  RenderScale,
  RenderTextureFiltering,
} from "@/core/ipc/types/xrf-renderer";
import { describeLevelFeatureToggle, ILevelFeatureOptions, LEVEL_ANTIALIASING_MODES } from "@/core/level/lib/features";
import { RenderValueChoice } from "@/core/render/components/controls/RenderValueChoice";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import {
  describeDlssPreset,
  describeDlssUnavailability,
  describeRenderAntialiasing,
  formatOutputScale,
  formatSharpening,
  IRenderChoiceOption,
  RENDER_DLSS_PRESET_SHORT_OPTIONS,
  RENDER_OUTPUT_SCALE_LIMITS,
  RENDER_RESOLUTION_OPTIONS,
  RENDER_SCALE_OPTIONS,
  RENDER_SHARPENING_LIMITS,
  RENDER_TEXTURE_FILTERING_SHORT_OPTIONS,
} from "@/core/render/lib/features";
import {
  describeRenderBackend,
  isRenderBackendAvailable,
  RENDER_BACKEND_AUTO,
  RENDER_BACKEND_CHOICES,
  TRenderBackendChoice,
} from "@/core/render/lib/settings/render-backend-choice";
import { ERenderResolution } from "@/core/render/lib/settings/render-resolution";
import { EditorPopoverToggle } from "@/core/shell/editor/EditorPopoverToggle";
import { BaseComponentProps } from "@/lib/dom/element-types";

const MODE_OPTIONS: ReadonlyArray<IRenderChoiceOption<RenderAntialiasing>> = LEVEL_ANTIALIASING_MODES.map(
  (value: RenderAntialiasing) => ({ label: describeRenderAntialiasing(value), value })
);

/** The modes offered where the GPU has no DLSS: DLSS greyed out, FSR 2 standing in for it. */
const MODE_OPTIONS_WITHOUT_DLSS: ReadonlyArray<IRenderChoiceOption<RenderAntialiasing>> = MODE_OPTIONS.map(
  (option: IRenderChoiceOption<RenderAntialiasing>) =>
    option.value === ERenderAntialiasing.DLSS ? { ...option, isDisabled: true } : option
);

/**
 * @param availability - What each graphics API can do here, none until known.
 * @returns The graphics APIs offered: automatic, then each by name, greyed out where this machine cannot draw with it.
 */
function toBackendOptions(
  availability: Nullable<ReadonlyArray<RenderBackendAvailability>>
): ReadonlyArray<IRenderChoiceOption<TRenderBackendChoice>> {
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

interface ILevelImageQualityActionProps extends BaseComponentProps {
  isOn: boolean;
  /** The mode the settings smooth with, which this view can only narrow to none. */
  settingsMode: RenderAntialiasing;
  /** How the settings filter textures, which this view may set over. */
  settingsTextureFiltering: RenderTextureFiltering;
  /** How much smaller than the output the settings draw, which this view may set over. */
  settingsScale: RenderScale;
  /** How sharply the settings finish a frame drawn smaller than the viewport, which this view may set over. */
  settingsSharpening: number;
  /** Which model the settings upscale DLSS with, which this view may set over. */
  settingsDlssPreset: RenderDlssPreset;
  /** What the settings resolve every viewport at, which this view may set over. */
  settingsResolution: ERenderResolution;
  /** The share of the viewport the settings resolve at while automatic, which this view may set over. */
  settingsOutputScale: number;
  /** The graphics API the settings draw every viewport with, which this popover sets for all of them. */
  backend: TRenderBackendChoice;
  /** What each graphics API can do on this machine, none until the renderer was asked. */
  backends: Nullable<ReadonlyArray<RenderBackendAvailability>>;
  /** Whether the view's frame is drawn smaller than the viewport, which sharpening needs to have any effect. */
  isUpscaled: boolean;
  /** Whether the GPU drawing the view upscales with DLSS; FSR 2 smooths in its place where it does not. */
  dlss: RenderDlssAvailability;
  features: ILevelFeatureOptions;
  onToggle: () => void;
  onChange: (features: ILevelFeatureOptions) => void;
  /** Draws every viewport with another graphics API, which restarts the renderer. */
  onChangeBackend: (backend: TRenderBackendChoice) => void;
}

/**
 * How this view's frame is made into the image it shows: whether its edges are smoothed and by which pass, how much
 * smaller it is drawn, what it is resolved at, and how its textures are sampled.
 */
export function LevelImageQualityAction({
  "data-testid": dataTestId = "level-image-quality-action",
  id,
  className,
  isOn,
  settingsMode,
  settingsTextureFiltering,
  settingsScale,
  settingsSharpening,
  settingsDlssPreset,
  settingsResolution,
  settingsOutputScale,
  backend,
  backends,
  isUpscaled,
  dlss,
  features,
  onToggle,
  onChange,
  onChangeBackend,
}: ILevelImageQualityActionProps): ReactElement {
  const isAvailable: boolean = settingsMode !== ERenderAntialiasing.NONE;
  const asked: RenderAntialiasing = features.antialiasing ?? settingsMode;
  const dlssProblem: Nullable<string> = describeDlssUnavailability(dlss);
  const mode: RenderAntialiasing = asked === ERenderAntialiasing.DLSS && dlssProblem ? ERenderAntialiasing.FSR2 : asked;
  const textureFiltering: RenderTextureFiltering = features.textureFiltering ?? settingsTextureFiltering;
  const scale: RenderScale = features.upscaling.scale ?? settingsScale;
  const sharpening: number = features.upscaling.sharpening ?? settingsSharpening;
  const dlssPreset: RenderDlssPreset = features.upscaling.dlssPreset ?? settingsDlssPreset;
  const resolution: ERenderResolution = features.output.resolution ?? settingsResolution;
  const outputScale: number = features.output.scale ?? settingsOutputScale;

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
            onChange={onChangeBackend}
          />

          <Typography className={"text-text-secondary"} variant={"caption"}>
            Every viewport draws with it; another one restarts the renderer.
          </Typography>
        </>
      }
      toggleLabel={"Antialiasing"}
      onToggle={onToggle}
    >
      <RenderValueChoice
        label={"Mode"}
        options={dlssProblem ? MODE_OPTIONS_WITHOUT_DLSS : MODE_OPTIONS}
        value={mode}
        onChange={(antialiasing: RenderAntialiasing) => onChange({ ...features, antialiasing })}
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
        onChange={(next: RenderScale) => onChange({ ...features, upscaling: { ...features.upscaling, scale: next } })}
      />

      {mode === ERenderAntialiasing.DLSS ? (
        <>
          <RenderValueChoice
            label={"DLSS model"}
            options={RENDER_DLSS_PRESET_SHORT_OPTIONS}
            value={dlssPreset}
            onChange={(next: RenderDlssPreset) =>
              onChange({ ...features, upscaling: { ...features.upscaling, dlssPreset: next } })
            }
          />

          <Typography className={"text-text-secondary"} variant={"caption"}>
            {describeDlssPreset(dlssPreset)}
          </Typography>
        </>
      ) : null}

      <RenderValueSlider
        label={"Sharpening"}
        value={sharpening}
        {...RENDER_SHARPENING_LIMITS}
        format={formatSharpening}
        isDisabled={!isUpscaled}
        onChange={(next: number) => onChange({ ...features, upscaling: { ...features.upscaling, sharpening: next } })}
      />

      {isUpscaled ? null : (
        <Typography className={"text-text-secondary"} variant={"caption"}>
          Sharpens only a frame drawn smaller than the viewport.
        </Typography>
      )}

      <RenderValueChoice
        label={"Output resolution"}
        options={RENDER_RESOLUTION_OPTIONS}
        value={resolution}
        onChange={(next: ERenderResolution) =>
          onChange({ ...features, output: { ...features.output, resolution: next } })
        }
      />

      {resolution === ERenderResolution.AUTO ? (
        <RenderValueSlider
          label={"Output scale"}
          value={outputScale}
          {...RENDER_OUTPUT_SCALE_LIMITS}
          format={formatOutputScale}
          onChange={(next: number) => onChange({ ...features, output: { ...features.output, scale: next } })}
        />
      ) : null}

      <RenderValueChoice
        label={"Texture filtering"}
        options={RENDER_TEXTURE_FILTERING_SHORT_OPTIONS}
        value={textureFiltering}
        onChange={(filtering: RenderTextureFiltering) => onChange({ ...features, textureFiltering: filtering })}
      />

      <Button
        size={"small"}
        onClick={() => onChange({ ...features, antialiasing: null, textureFiltering: null, upscaling: {}, output: {} })}
      >
        Back to the settings
      </Button>
    </EditorPopoverToggle>
  );
}
