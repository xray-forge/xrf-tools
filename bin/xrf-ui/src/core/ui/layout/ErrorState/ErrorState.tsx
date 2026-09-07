import { default as ErrorOutlineIcon } from "@mui/icons-material/ErrorOutlineOutlined";
import { Box, Button } from "@mui/material";
import { ReactElement } from "react";

import { EmptyState } from "@/core/ui/layout/EmptyState";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IErrorStateProps extends BaseComponentProps {
  title: string;
  description: string;
  /** Repeats the failed request through its owner. Omit when the surface cannot retry. */
  onRetry?: () => void;
}

/** Announces failed content with consistent layout and an optional retry action. */
export function ErrorState({
  "data-testid": dataTestId,
  id,
  className,
  title,
  description,
  onRetry,
}: IErrorStateProps): ReactElement {
  return (
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      role={"alert"}
      sx={{ width: "100%", height: "100%", minWidth: 0 }}
    >
      <EmptyState
        title={title}
        description={description}
        icon={<ErrorOutlineIcon sx={{ color: "error.main" }} />}
        action={
          onRetry ? (
            <Button variant={"outlined"} onClick={onRetry}>
              Retry
            </Button>
          ) : undefined
        }
      />
    </Box>
  );
}
