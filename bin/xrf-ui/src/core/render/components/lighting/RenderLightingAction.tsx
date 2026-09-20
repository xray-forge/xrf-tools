import { default as WbSunnyIcon } from "@mui/icons-material/WbSunny";
import { Button, Typography } from "@mui/material";
import { ReactElement } from "react";

import { RenderLightingControls } from "@/core/render/components/lighting/RenderLightingControls";
import { IRenderLighting } from "@/core/render/lib/lighting/render-lighting";
import { EditorPopoverAction } from "@/core/shell/editor/EditorPopoverAction";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IRenderLightingActionProps extends BaseComponentProps {
  lighting: IRenderLighting;
  /** What the reset button puts back, which each surface answers for itself. */
  fallback: IRenderLighting;
  isDisabled?: boolean;
  /** Said in place of the summary while the action is disabled, so it reads as a reason rather than a state. */
  unavailableDescription?: string;
  onChange: (lighting: IRenderLighting) => void;
}

/**
 * The lighting controls in a toolbar, for a preview whose panels are not the place for them.
 */
export function RenderLightingAction({
  "data-testid": dataTestId = "render-lighting-action",
  id,
  className,
  lighting,
  fallback,
  isDisabled = false,
  unavailableDescription,
  onChange,
}: IRenderLightingActionProps): ReactElement {
  return (
    <EditorPopoverAction
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Lighting"}
      description={
        isDisabled && unavailableDescription
          ? unavailableDescription
          : `Lighting: ${lighting.sunElevation}° up, ${lighting.sunIntensity.toFixed(2)} light`
      }
      icon={<WbSunnyIcon />}
      isDisabled={isDisabled}
    >
      <div className={"flex w-56 flex-col gap-2 px-4 py-2"}>
        <Typography className={"text-text-secondary"} variant={"overline"}>
          Lighting
        </Typography>

        <RenderLightingControls lighting={lighting} onChange={onChange} />

        <Button size={"small"} onClick={() => onChange(fallback)}>
          Back to the default light
        </Button>
      </div>
    </EditorPopoverAction>
  );
}
