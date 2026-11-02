import { isFlowCancellationError } from "@wirestate/mobx";

/**
 * Reports whether a rejection is a flow that was cancelled rather than a failure.
 *
 * @param error - Rejection to classify.
 * @returns Whether the flow was cancelled.
 */
export function isCancellation(error: unknown): boolean {
  return isFlowCancellationError(error as Error);
}

/**
 * Lets a cancellation settle quietly while a real failure still reaches the caller.
 *
 * @param error - Rejection the flow produced.
 */
export function swallowCancellation(error: unknown): void {
  if (!isFlowCancellationError(error as Error)) {
    throw error;
  }
}
