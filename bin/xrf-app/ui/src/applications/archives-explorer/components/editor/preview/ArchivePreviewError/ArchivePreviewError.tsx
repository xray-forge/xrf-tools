import { BaseComponentProps } from "@xrf/ui/element-types";
import { ErrorOutlineOutlinedIcon as ErrorOutlineIcon } from "@xrf/ui/icons";
import { CenteredColumn } from "@xrf/ui/layout/CenteredColumn";
import { Button, Typography } from "@xrf/ui/material";
import { cn } from "@xrf/ui/theme/class-name";
import { ReactElement } from "react";

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
      <ErrorOutlineIcon className={"text-content-state-icon"} color={"error"} />

      <Typography variant={"subtitle1"}>Could not read this file</Typography>

      <Typography variant={"body2"} className={"max-w-130 whitespace-pre-line text-text-secondary"}>
        {error.message}
      </Typography>

      <Button variant={"outlined"} onClick={onRetry}>
        Retry
      </Button>
    </CenteredColumn>
  );
}
