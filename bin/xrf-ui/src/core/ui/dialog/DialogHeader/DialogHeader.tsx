import { default as CloseIcon } from "@mui/icons-material/Close";
import { Box, DialogTitle, IconButton, Tooltip, Typography } from "@mui/material";
import { ReactElement, ReactNode } from "react";

import { DIALOG } from "@/core/theme/tokens";
import { BaseComponentProps } from "@/lib/dom/element-types";

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
      className={className}
      component={"div"}
      sx={{ display: "flex", alignItems: "center", gap: DIALOG.gap }}
    >
      {icon ? (
        <Box component={"span"} aria-hidden={true} sx={{ display: "inline-flex" }}>
          {icon}
        </Box>
      ) : null}

      <Typography
        id={titleId}
        component={"h2"}
        variant={"h6"}
        sx={{ flexGrow: 1, minWidth: 0, overflowWrap: "anywhere" }}
      >
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
