import { default as ErrorOutlineIcon } from "@mui/icons-material/ErrorOutlineOutlined";
import { Box, Button } from "@mui/material";
import { ReactElement } from "react";

import { EmptyState } from "@/core/ui/layout/EmptyState";
import { BaseComponentProps } from "@/lib/dom/element-types";

export interface IVisualPreviewEmptyProps extends BaseComponentProps {
  /** Why the last open failed, or absent when nothing is open and nothing went wrong. */
  error?: string;
  /** Reads the failed open's source again. Absent where the surface has no attempt to repeat. */
  onRetry?: () => void;
}

/**
 * What covers the viewport while it holds no model: nothing has been opened, or the last open failed.
 */
export function VisualPreviewEmpty({ error, onRetry }: IVisualPreviewEmptyProps): ReactElement {
  return (
    <Box sx={{ position: "absolute", inset: 0, display: "flex", backgroundColor: "background.default" }}>
      <EmptyState
        title={error ? "Could not open this visual" : "No visual open"}
        description={error ?? "Pick a model from the tree to preview it."}
        icon={error ? <ErrorOutlineIcon /> : undefined}
        action={
          error && onRetry ? (
            <Button variant={"outlined"} onClick={() => void onRetry()}>
              Retry
            </Button>
          ) : undefined
        }
      />
    </Box>
  );
}
