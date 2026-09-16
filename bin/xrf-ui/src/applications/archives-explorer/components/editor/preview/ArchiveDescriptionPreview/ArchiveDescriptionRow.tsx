import { Typography } from "@mui/material";
import { ReactElement, ReactNode } from "react";

import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IArchiveDescriptionRowProps extends BaseComponentProps {
  label: string;
  value: ReactNode;
  /** What qualifies the value: where a number came from, what a flag switches on. */
  caption?: ReactNode;
  /** Uses the shared identifier and path typography for the value. */
  isMonospace?: boolean;
}

/**
 * One labelled fact of a description.
 */
export function ArchiveDescriptionRow({
  "data-testid": dataTestId = "archive-description-row",
  id,
  className,
  label,
  value,
  caption,
  isMonospace = false,
}: IArchiveDescriptionRowProps): ReactElement {
  return (
    <dl
      data-testid={dataTestId}
      id={id}
      className={cn(
        "m-0 grid min-w-0 grid-cols-1 gap-0.5 py-1.5 leading-panel",
        "sm:grid-cols-[var(--container-reading-label)_1fr]",
        className
      )}
    >
      <Typography
        className={"min-w-0 leading-[inherit] wrap-anywhere text-text-secondary"}
        component={"dt"}
        variant={"body2"}
      >
        {label}
      </Typography>

      <dd className={"m-0 min-w-0"}>
        <Typography
          className={cn("min-w-0 leading-[inherit] wrap-anywhere", isMonospace && "monospace")}
          component={"div"}
          variant={"body2"}
        >
          {value}
        </Typography>

        {caption ? (
          <Typography className={"block wrap-anywhere text-text-disabled"} component={"div"} variant={"caption"}>
            {caption}
          </Typography>
        ) : null}
      </dd>
    </dl>
  );
}
