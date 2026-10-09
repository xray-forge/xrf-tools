import { default as HighQualityIcon } from "@mui/icons-material/HighQuality";
import { Button, Typography } from "@mui/material";
import { Nullable } from "@xrf/types";
import { ReactElement } from "react";

import {
  ERenderAntialiasing,
  RenderAntialiasing,
  RenderDlssAvailability,
  RenderScale,
  RenderTextureFiltering,
} from "@/core/ipc/types/xrf-renderer";
import { describeLevelFeatureToggle, ILevelFeatureOptions, LEVEL_ANTIALIASING_MODES } from "@/core/level/lib/features";
import { RenderValueChoice } from "@/core/render/components/controls/RenderValueChoice";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import {
  describeDlssUnavailability,
  describeRenderAntialiasing,
  formatOutputScale,
  IRenderChoiceOption,
  RENDER_OUTPUT_SCALE_LIMITS,
  RENDER_RESOLUTION_OPTIONS,
  RENDER_SCALE_OPTIONS,
  RENDER_TEXTURE_FILTERING_SHORT_OPTIONS,
} from "@/core/render/lib/features";
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

interface ILevelImageQualityActionProps extends BaseComponentProps {
  isOn: boolean;
  /** The mode the settings smooth with, which this view can only narrow to none. */
  settingsMode: RenderAntialiasing;
  /** How the settings filter textures, which this view may set over. */
  settingsTextureFiltering: RenderTextureFiltering;
  /** How much smaller than the output the settings draw, which this view may set over. */
  settingsScale: RenderScale;
  /** What the settings resolve every viewport at, which this view may set over. */
  settingsResolution: ERenderResolution;
  /** The share of the viewport the settings resolve at while automatic, which this view may set over. */
  settingsOutputScale: number;
  /** Whether the GPU drawing the view upscales with DLSS; FSR 2 smooths in its place where it does not. */
  dlss: RenderDlssAvailability;
  features: ILevelFeatureOptions;
  onToggle: () => void;
  onChange: (features: ILevelFeatureOptions) => void;
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
  settingsResolution,
  settingsOutputScale,
  dlss,
  features,
  onToggle,
  onChange,
}: ILevelImageQualityActionProps): ReactElement {
  const isAvailable: boolean = settingsMode !== ERenderAntialiasing.NONE;
  const asked: RenderAntialiasing = features.antialiasing ?? settingsMode;
  const dlssProblem: Nullable<string> = describeDlssUnavailability(dlss);
  const mode: RenderAntialiasing = asked === ERenderAntialiasing.DLSS && dlssProblem ? ERenderAntialiasing.FSR2 : asked;
  const textureFiltering: RenderTextureFiltering = features.textureFiltering ?? settingsTextureFiltering;
  const scale: RenderScale = features.upscaling.scale ?? settingsScale;
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
      toggleLabel={"Smooth the frame's edges"}
      onToggle={onToggle}
    >
      <RenderValueChoice
        label={"Antialiasing"}
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
        onChange={(next: RenderScale) => onChange({ ...features, upscaling: { scale: next } })}
      />

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
