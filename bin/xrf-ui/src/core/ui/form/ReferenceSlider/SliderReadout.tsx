import { default as RestartAltIcon } from "@mui/icons-material/RestartAlt";
import { IconButton, Tooltip, Typography, TypographyProps } from "@mui/material";
import { Nullable } from "@xrf/types";
import { ReactElement } from "react";

import { describeSliderRecommendation, ISliderReference } from "@/core/ui/form/ReferenceSlider/slider-reference";
import { BaseComponentProps } from "@/lib/dom/element-types";

export interface ISliderReadoutProps extends BaseComponentProps {
  value: number;
  format: (value: number) => string;
  reference: Nullable<ISliderReference>;
  recommended?: Nullable<readonly [number, number]>;
  /** The reading's type, the popovers' compact one or the settings' body. */
  variant?: TypographyProps["variant"];
  isDisabled?: boolean;
  onReset: (value: number) => void;
}

/**
 * A slider's value as it reads: amber outside the recommended range, saying why on hover, and a reset to the reference
 * beside it once the value has left it.
 */
export function SliderReadout({
  "data-testid": dataTestId = "slider-readout",
  id,
  className,
  value,
  format,
  reference,
  recommended,
  variant = "caption",
  isDisabled = false,
  onReset,
}: ISliderReadoutProps): ReactElement {
  const warning: Nullable<string> = describeSliderRecommendation(value, recommended);
  const referenceValue: Nullable<number> = reference?.value ?? null;
  const isChanged: boolean = referenceValue !== null && format(referenceValue) !== format(value);

  return (
    <span data-testid={dataTestId} id={id} className={`inline-flex items-center gap-0.5 ${className ?? ""}`}>
      {isChanged && referenceValue !== null && reference ? (
        <Tooltip title={`Reset to ${reference.label}: ${format(referenceValue)}`}>
          <span>
            <IconButton
              size={"small"}
              aria-label={`Reset to ${reference.label}: ${format(referenceValue)}`}
              disabled={isDisabled}
              sx={{ padding: "1px" }}
              onClick={() => onReset(referenceValue)}
            >
              <RestartAltIcon sx={{ fontSize: 14 }} />
            </IconButton>
          </span>
        </Tooltip>
      ) : null}

      <Tooltip title={warning ?? ""} disableHoverListener={!warning}>
        <Typography
          component={"span"}
          variant={variant}
          className={"tabular-nums"}
          sx={{
            color: warning ? "warning.main" : "text.primary",
            fontFamily: variant === "caption" ? "monospace" : undefined,
          }}
        >
          {format(value)}
        </Typography>
      </Tooltip>
    </span>
  );
}
