import { formatDegrees, formatNumber } from "@xrf/format";
import { EditorPopoverAction } from "@xrf/ui/editor/popover-action";
import { EditorPopoverContent } from "@xrf/ui/editor/popover-content";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { WbSunnyIcon } from "@xrf/ui/icons/wb-sunny";
import { ReactElement, useCallback } from "react";

import { RenderLightingControls } from "@/core/render/components/lighting/RenderLightingControls";
import { IRenderLighting, isSameRenderLighting } from "@/core/render/lib/lighting/render-lighting";

interface IRenderLightingActionProps extends BaseComponentProps {
  lighting: IRenderLighting;
  /** What the heading's reset puts back, offered while the light differs from it; each surface answers for itself. */
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
  const onReset = useCallback(() => onChange(fallback), [fallback, onChange]);

  return (
    <EditorPopoverAction
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Lighting"}
      description={
        isDisabled && unavailableDescription
          ? unavailableDescription
          : `Sun ${formatDegrees(lighting.sunElevation)} up, ${formatNumber(lighting.sunIntensity, 2)} light`
      }
      icon={<WbSunnyIcon />}
      isDisabled={isDisabled}
    >
      <EditorPopoverContent
        label={"Lighting"}
        resetLabel={"Back to the default light"}
        onReset={isSameRenderLighting(lighting, fallback) ? undefined : onReset}
      >
        <RenderLightingControls lighting={lighting} onChange={onChange} />
      </EditorPopoverContent>
    </EditorPopoverAction>
  );
}
