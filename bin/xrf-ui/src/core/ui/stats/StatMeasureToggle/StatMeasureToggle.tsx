import { ToggleButton, ToggleButtonGroup } from "@mui/material";
import { ReactElement, useCallback } from "react";

import { EStatMeasure } from "@/core/ui/stats/stat-measure";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

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
      <ToggleButton value={EStatMeasure.BYTES} sx={{ paddingY: 0.25, paddingX: 1, textTransform: "none" }}>
        Bytes
      </ToggleButton>

      <ToggleButton value={EStatMeasure.COUNT} sx={{ paddingY: 0.25, paddingX: 1, textTransform: "none" }}>
        Count
      </ToggleButton>
    </ToggleButtonGroup>
  );
}
