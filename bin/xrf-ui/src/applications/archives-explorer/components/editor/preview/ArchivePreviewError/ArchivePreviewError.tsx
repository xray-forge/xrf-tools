import { default as ErrorOutlineIcon } from "@mui/icons-material/ErrorOutlineOutlined";
import { Button, Typography } from "@mui/material";
import { ReactElement } from "react";

import { CenteredColumn } from "@/core/ui/layout/CenteredColumn";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IArchivePreviewErrorProps extends BaseComponentProps {
  error: Error;
  onRetry: () => void;
}

export function ArchivePreviewError({ error, onRetry }: IArchivePreviewErrorProps): ReactElement {
  return (
    <CenteredColumn sx={{ padding: 3, textAlign: "center" }}>
      <ErrorOutlineIcon color={"error"} sx={{ fontSize: 40 }} />

      <Typography variant={"subtitle1"}>Could not read this file</Typography>

      <Typography variant={"body2"} sx={{ maxWidth: 520, color: "text.secondary", whiteSpace: "pre-line" }}>
        {error.message}
      </Typography>

      <Button variant={"outlined"} onClick={() => void onRetry()}>
        Retry
      </Button>
    </CenteredColumn>
  );
}
