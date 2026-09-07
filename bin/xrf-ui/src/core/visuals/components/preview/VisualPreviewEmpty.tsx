import { Box } from "@mui/material";
import { ReactElement } from "react";

import { EmptyState } from "@/core/ui/layout/EmptyState";
import { ErrorState } from "@/core/ui/layout/ErrorState";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IVisualPreviewEmptyProps extends BaseComponentProps {
  /** Why the last open failed, or absent when nothing is open and nothing went wrong. */
  error?: string;
  /** Reads the failed open's source again. Absent where the surface has no attempt to repeat. */
  onRetry?: () => void;
}

/**
 * What covers the viewport while it holds no model: nothing has been opened, or the last open failed.
 */
export function VisualPreviewEmpty({
  "data-testid": dataTestId,
  id,
  className,
  error,
  onRetry,
}: IVisualPreviewEmptyProps): ReactElement {
  return (
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={{ position: "absolute", inset: 0, display: "flex", backgroundColor: "background.default" }}
    >
      {error ? (
        <ErrorState title={"Could not open this visual"} description={error} onRetry={onRetry} />
      ) : (
        <EmptyState title={"No visual open"} description={"Pick a model from the tree to preview it."} />
      )}
    </Box>
  );
}
