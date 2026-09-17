import { Tooltip, Typography } from "@mui/material";
import { ReactElement } from "react";

import { cn } from "@/lib/dom/dom-name";

interface IEditorToolbarCrumbProps {
  label: string;
  isDisabled?: boolean;
  /** Says what following the segment does, for a name the label alone cannot carry. */
  accessibleName?: string;
  /** Describes what following the segment does. Only meaningful when it is interactive. */
  hint?: string;
  onClick?: () => void;
}

/**
 * One segment of the caption's breadcrumb.
 */
export function EditorToolbarCrumb({
  isDisabled,
  label,
  accessibleName,
  hint,
  onClick,
}: IEditorToolbarCrumbProps): ReactElement {
  if (!onClick) {
    return (
      <Typography className={"shrink-0 font-semibold"} variant={"subtitle2"} noWrap={true}>
        {label}
      </Typography>
    );
  }

  const crumb: ReactElement = (
    <Typography
      aria-label={accessibleName}
      noWrap={true}
      component={"button"}
      disabled={isDisabled}
      variant={"subtitle2"}
      className={cn(
        "shrink-0 appearance-none rounded-control border-0 bg-transparent px-1.25 py-0.5 font-semibold",
        "focus-visible:text-text-primary focus-visible:outline-1 focus-visible:outline-offset-1 focus-visible:outline-primary",
        isDisabled
          ? "cursor-default text-text-disabled"
          : "cursor-pointer text-text-secondary hover:bg-action-hover hover:text-text-primary"
      )}
      onClick={onClick}
    >
      {label}
    </Typography>
  );

  return hint ? (
    <Tooltip describeChild title={hint}>
      <span>{crumb}</span>
    </Tooltip>
  ) : (
    crumb
  );
}
