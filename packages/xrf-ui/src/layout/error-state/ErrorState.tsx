import { ReactElement } from "react";

import { BaseComponentProps } from "#/element-types";
import { ErrorOutlineOutlinedIcon as ErrorOutlineIcon } from "#/icons/error-outline-outlined";
import { EmptyState } from "#/layout/empty-state";
import { Button } from "#/material/button";
import { cn } from "#/theme";

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
    <div data-testid={dataTestId} id={id} className={cn("h-full w-full min-w-0", className)} role={"alert"}>
      <EmptyState
        title={title}
        description={description}
        icon={<ErrorOutlineIcon className={"text-error"} />}
        action={
          onRetry ? (
            <Button variant={"outlined"} onClick={onRetry}>
              Retry
            </Button>
          ) : undefined
        }
      />
    </div>
  );
}
