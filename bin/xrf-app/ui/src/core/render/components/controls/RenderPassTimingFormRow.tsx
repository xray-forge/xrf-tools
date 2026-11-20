import { BaseComponentProps } from "@xrf/ui/element-types";
import { CheckboxFormRow } from "@xrf/ui/form/checkbox-row";
import { ReactElement } from "react";

/** What the switch for timing every pass on the GPU is called, wherever it is offered. */
export const RENDER_PASS_TIMING_LABEL: string = "GPU time per pass";

/** What timing every pass does and costs, wherever its switch is offered. */
export const RENDER_PASS_TIMING_DESCRIPTION: string =
  "Shows each pass's GPU time in the readout. Costs 1-5% of the frame rate.";

export interface IRenderPassTimingFormRowProps extends BaseComponentProps {
  /** Whether the settings time every pass on the GPU. */
  isChecked: boolean;
  onChange: (isGpuTimed: boolean) => void;
}

/**
 * The settings' one switch for timing every pass on the GPU, the same row wherever it is offered.
 */
export function RenderPassTimingFormRow({
  "data-testid": dataTestId = "render-pass-timing-form-row",
  id,
  className,
  isChecked,
  onChange,
}: IRenderPassTimingFormRowProps): ReactElement {
  return (
    <CheckboxFormRow
      data-testid={dataTestId}
      id={id}
      className={className}
      label={RENDER_PASS_TIMING_LABEL}
      description={RENDER_PASS_TIMING_DESCRIPTION}
      isChecked={isChecked}
      onChange={onChange}
    />
  );
}
