import { IconButton, svgIconClasses, Tooltip, useTheme } from "@mui/material";
import { ReactElement, ReactNode, useId } from "react";

import { getControlStateSx } from "@/core/theme/control-state";

import { RAIL_BUTTON_SX } from "./RailButton.styles";

interface IRailButtonProps {
  isSelected?: boolean;
  isDisabled?: boolean;
  /** Stable accessible name, independent of what the control can do right now. */
  label: string;
  /** Tooltip text; defaults to the label. Say here why a disabled control is disabled. */
  description?: string;
  icon: ReactNode;
  appearance?: "gradient" | "primary" | "secondary";
  onClick: () => void;
}

/**
 * A control on the rail that acts rather than opening a panel.
 */
export function RailButton({
  isSelected,
  isDisabled,
  label,
  description = label,
  icon,
  appearance = "gradient",
  onClick,
}: IRailButtonProps): ReactElement {
  const gradientId: string = useId();
  const theme = useTheme();

  const palette = (theme.vars ?? theme).palette;
  const isGradient: boolean = appearance === "gradient";

  return (
    <Tooltip describeChild title={description} placement={"right"}>
      <span>
        <IconButton
          aria-label={label}
          disabled={isDisabled}
          style={{ fill: isDisabled || !isGradient ? "currentColor" : `url("#${gradientId}")` }}
          sx={[
            RAIL_BUTTON_SX,
            getControlStateSx(Boolean(isSelected)),
            {
              [`& .${svgIconClasses.root}`]: { fill: "inherit" },
              ...(isGradient ? {} : { color: `${appearance}.main`, backgroundImage: "none" }),
            },
          ]}
          onClick={onClick}
        >
          {isGradient ? (
            <svg aria-hidden={true} width={0} height={0} focusable={false}>
              <defs>
                <linearGradient id={gradientId} x1={"0%"} y1={"0%"} x2={"100%"} y2={"0%"}>
                  <stop offset={"0%"} stopColor={palette.primary.main} />
                  <stop offset={"100%"} stopColor={palette.secondary.main} />
                </linearGradient>
              </defs>
            </svg>
          ) : null}

          {icon}
        </IconButton>
      </span>
    </Tooltip>
  );
}
