import { default as RestartAltIcon } from "@mui/icons-material/RestartAlt";
import { IconButton, Tooltip, Typography } from "@mui/material";
import { Nullable } from "@xrf/types";
import { ReactElement, useCallback } from "react";

import { cn } from "@/core/theme/class-name";
import { SLIDER_REFERENCE } from "@/core/theme/tokens";
import {
  describeSliderRecommendation,
  ISliderReference,
  isSliderWasted,
} from "@/core/ui/form/ReferenceSlider/slider-reference";
import { BaseComponentProps } from "@/lib/dom/element-types";

export interface ISliderReadoutProps extends BaseComponentProps {
  value: number;
  format: (value: number) => string;
  reference: Nullable<ISliderReference>;
  recommended?: Nullable<readonly [number, number]>;
  wastedAbove?: Nullable<number>;
  /** Whether it reads in the popovers' compact monospace rather than the settings' body text. */
  isCompact?: boolean;
  isDisabled?: boolean;
  onReset: (value: number) => void;
}

/**
 * A slider's value as it reads: in the warning's colour outside the recommended range and the error's past the useful
 * end, saying why on hover, and a reset
 * to the reference beside it once the value has left it.
 */
export function SliderReadout({
  "data-testid": dataTestId = "slider-readout",
  id,
  className,
  value,
  format,
  reference,
  recommended,
  wastedAbove,
  isCompact = false,
  isDisabled = false,
  onReset,
}: ISliderReadoutProps): ReactElement {
  const warning: Nullable<string> = describeSliderRecommendation(value, recommended, wastedAbove);
  const isWasted: boolean = isSliderWasted(value, wastedAbove);
  const referenceValue: Nullable<number> = reference?.value ?? null;
  const resetLabel: Nullable<string> =
    reference && referenceValue !== null && format(referenceValue) !== format(value)
      ? `Reset to ${reference.label}: ${format(referenceValue)}`
      : null;

  const onResetClick = useCallback(() => {
    if (referenceValue !== null) {
      onReset(referenceValue);
    }
  }, [referenceValue, onReset]);

  return (
    <span data-testid={dataTestId} id={id} className={cn("inline-flex items-center gap-0.5", className)}>
      {resetLabel ? (
        <Tooltip title={resetLabel}>
          <span className={"inline-flex"}>
            <IconButton
              size={"small"}
              aria-label={resetLabel}
              disabled={isDisabled}
              sx={{ padding: `${SLIDER_REFERENCE.resetPadding}px` }}
              onClick={onResetClick}
            >
              <RestartAltIcon sx={{ fontSize: SLIDER_REFERENCE.resetIconSize }} />
            </IconButton>
          </span>
        </Tooltip>
      ) : null}

      <Tooltip title={warning ?? ""} disableHoverListener={!warning}>
        <Typography
          component={"span"}
          variant={isCompact ? "caption" : "body2"}
          className={cn("tabular-nums", isCompact && "font-mono")}
          sx={{ color: isWasted ? "error.main" : warning ? "warning.main" : "text.primary" }}
        >
          {format(value)}
        </Typography>
      </Tooltip>
    </span>
  );
}
