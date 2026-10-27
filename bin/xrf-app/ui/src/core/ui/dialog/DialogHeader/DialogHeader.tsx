import { BaseComponentProps } from "@xrf/ui/element-types";
import { CloseIcon } from "@xrf/ui/icons";
import { DialogTitle, IconButton, Tooltip, Typography } from "@xrf/ui/material";
import { cn } from "@xrf/ui/theme/class-name";
import { ReactElement, ReactNode } from "react";

export interface IDialogHeaderProps extends BaseComponentProps {
  title: string;
  /** Identifies only the heading text for the dialog's aria-labelledby attribute. */
  titleId: string;
  icon?: ReactNode;
  closeLabel?: string;
  /** Omit when dismissal is already offered by the dialog's actions. */
  onClose?: () => void;
}

/** Shared dialog heading, with an optional decorative icon and named close action. */
export function DialogHeader({
  "data-testid": dataTestId = "dialog-header",
  id,
  className,
  title,
  titleId,
  icon,
  closeLabel = "Close dialog",
  onClose,
}: IDialogHeaderProps): ReactElement {
  return (
    <DialogTitle
      data-testid={dataTestId}
      id={id}
      className={cn("flex items-center gap-2", className)}
      component={"div"}
    >
      {icon ? (
        <span aria-hidden={true} className={"inline-flex"}>
          {icon}
        </span>
      ) : null}

      <Typography id={titleId} className={"min-w-0 grow wrap-anywhere"} component={"h2"} variant={"h6"}>
        {title}
      </Typography>

      {onClose ? (
        <Tooltip title={closeLabel}>
          <IconButton aria-label={closeLabel} size={"small"} onClick={onClose}>
            <CloseIcon fontSize={"small"} />
          </IconButton>
        </Tooltip>
      ) : null}
    </DialogTitle>
  );
}
