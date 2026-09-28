import { ReactElement } from "react";

import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { BaseComponentProps } from "@/lib/dom/element-types";

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
      label={"GPU time per pass"}
      description={"Lists each pass's GPU cost in every viewer's readout. Timing costs 1-4% of the frame rate."}
      isChecked={isChecked}
      onChange={onChange}
    />
  );
}
