import { default as AspectRatioIcon } from "@mui/icons-material/AspectRatio";
import { Typography } from "@mui/material";
import { ERendererRenderScale } from "@xrf/renderer";
import { ReactElement } from "react";

import { RenderValueChoice } from "@/core/render/components/controls/RenderValueChoice";
import { describeRenderScale, RENDER_SCALE_OPTIONS } from "@/core/render/lib/features";
import { EditorPopoverAction } from "@/core/shell/editor/EditorPopoverAction";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ILevelRenderScaleActionProps extends BaseComponentProps {
  /** The render scale in the settings, which every viewport draws at. */
  scale: ERendererRenderScale;
  /** Sets it in the settings, for every viewport. */
  onChange: (scale: ERendererRenderScale) => void;
}

/**
 * How much of each side the scene is drawn at before it is upscaled: the settings' own, every viewport's, whatever the
 * antialiasing, since the modes that do not upscale themselves are upscaled by FSR 1.
 */
export function LevelRenderScaleAction({
  "data-testid": dataTestId = "level-render-scale-action",
  id,
  className,
  scale,
  onChange,
}: ILevelRenderScaleActionProps): ReactElement {
  const isUpscaled: boolean = scale !== ERendererRenderScale.NATIVE;

  return (
    <EditorPopoverAction
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Render scale"}
      description={
        isUpscaled
          ? `Every viewport drawn at ${describeRenderScale(scale)} and upscaled`
          : "Every viewport drawn at its own size"
      }
      icon={<AspectRatioIcon />}
      isActive={isUpscaled}
    >
      <div className={"flex w-max min-w-60 flex-col gap-2 px-4 py-2"}>
        <Typography className={"text-text-secondary"} variant={"overline"}>
          Render scale
        </Typography>

        <RenderValueChoice
          label={"Every viewport's render scale"}
          options={RENDER_SCALE_OPTIONS}
          value={scale}
          onChange={onChange}
        />
      </div>
    </EditorPopoverAction>
  );
}
