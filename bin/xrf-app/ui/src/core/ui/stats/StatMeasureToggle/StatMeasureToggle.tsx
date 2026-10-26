import { ToggleButton, ToggleButtonGroup } from "@mui/material";
import { Nullable } from "@xrf/types";
import { ReactElement, useCallback } from "react";

import { EStatMeasure } from "@/core/ui/stats/stat-measure";
import { BaseComponentProps } from "@/lib/dom/element-types";

export interface IStatMeasureToggleProps extends BaseComponentProps {
  measure: EStatMeasure;
  onChange: (measure: EStatMeasure) => void;
}

/**
 * Chooses which measurement a breakdown is ordered and drawn by.
 */
export function StatMeasureToggle({
  "data-testid": dataTestId = "stat-measure-toggle",
  id,
  className,
  measure,
  onChange,
}: IStatMeasureToggleProps): ReactElement {
  const onSelect = useCallback(
    (_: unknown, next: Nullable<EStatMeasure>) => {
      if (next) {
        onChange(next);
      }
    },
    [onChange]
  );

  return (
    <ToggleButtonGroup
      data-testid={dataTestId}
      id={id}
      className={className}
      exclusive
      size={"small"}
      value={measure}
      aria-label={"Measure breakdowns by"}
      onChange={onSelect}
    >
      <ToggleButton className={"px-2 py-0.5 normal-case"} value={EStatMeasure.BYTES}>
        Bytes
      </ToggleButton>

      <ToggleButton className={"px-2 py-0.5 normal-case"} value={EStatMeasure.COUNT}>
        Count
      </ToggleButton>
    </ToggleButtonGroup>
  );
}
