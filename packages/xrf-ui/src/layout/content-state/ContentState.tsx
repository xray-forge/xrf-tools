import { Nullable } from "@xrf/types";
import { ReactElement, ReactNode } from "react";

import { BaseComponentProps } from "#/element-types";
import { DelayedProgress } from "#/layout/delayed-progress";
import { EmptyState } from "#/layout/empty-state";
import { ErrorState } from "#/layout/error-state";

export interface IContentStateProps<T> extends BaseComponentProps {
  /** What there is to show, or null while there is nothing. A value held through a reload stays on screen. */
  value: Nullable<T>;
  /** Whether a read is under way; shown only while there is no value to keep on screen. */
  isLoading?: boolean;
  /** Why the last read failed, which replaces everything else, a held value included. */
  error?: Nullable<Error>;
  /** Names the pending read. */
  loadingLabel?: string;
  errorTitle: string;
  /** Repeats the failed read through its owner. Omit when the surface cannot retry. */
  onRetry?: () => void;
  /** What a finished read that found nothing is called. */
  emptyTitle: string;
  emptyDescription: string;
  /** Draws the value; the props above only reach the states standing in for it. */
  children: (value: T) => ReactNode;
}

/**
 * One read's content, or what stands in for it: the failure, the pending read, or that there is nothing.
 */
export function ContentState<T>({
  "data-testid": dataTestId,
  id,
  className,
  value,
  isLoading = false,
  error = null,
  loadingLabel,
  errorTitle,
  onRetry,
  emptyTitle,
  emptyDescription,
  children,
}: IContentStateProps<T>): ReactElement {
  if (error) {
    return (
      <ErrorState
        data-testid={dataTestId}
        id={id}
        className={className}
        title={errorTitle}
        description={error.message}
        onRetry={onRetry}
      />
    );
  }

  if (value !== null) {
    return <>{children(value)}</>;
  }

  if (isLoading) {
    return <DelayedProgress data-testid={dataTestId} id={id} className={className} label={loadingLabel} />;
  }

  return (
    <EmptyState
      data-testid={dataTestId}
      id={id}
      className={className}
      title={emptyTitle}
      description={emptyDescription}
    />
  );
}
