import { default as ErrorOutlineIcon } from "@mui/icons-material/ErrorOutlineOutlined";
import { Button, Typography } from "@mui/material";
import { ReactElement } from "react";

import { CenteredColumn } from "@/core/ui/layout/CenteredColumn";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IArchivePreviewErrorProps extends BaseComponentProps {
  error: Error;
  onRetry: () => void;
}

export function ArchivePreviewError({
  "data-testid": dataTestId,
  id,
  className,
  error,
  onRetry,
}: IArchivePreviewErrorProps): ReactElement {
  return (
    <CenteredColumn data-testid={dataTestId} id={id} className={cn("p-6 text-center", className)}>
      <ErrorOutlineIcon color={"error"} sx={{ fontSize: 40 }} />

      <Typography variant={"subtitle1"}>Could not read this file</Typography>

      <Typography variant={"body2"} className={"max-w-130 whitespace-pre-line text-text-secondary"}>
        {error.message}
      </Typography>

      <Button variant={"outlined"} onClick={() => void onRetry()}>
        Retry
      </Button>
    </CenteredColumn>
  );
}
