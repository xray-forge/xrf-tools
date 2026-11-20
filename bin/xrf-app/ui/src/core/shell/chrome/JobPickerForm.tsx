import { Nullable } from "@xrf/types";
import { ReactElement, ReactNode, useCallback, useEffect } from "react";

import { JobProgressView } from "@/core/jobs/components/JobProgressView";
import { IJobState } from "@/core/jobs/lib";
import { JobOperation } from "@/core/jobs/lib/job-operation";
import { IPickerFormProps, PickerForm } from "@/core/shell/chrome/PickerForm";

export interface IJobPickerFormProps<TRequest, TResult> extends Omit<
  IPickerFormProps,
  "error" | "status" | "result" | "onSubmit"
> {
  /** The run the form starts, whose progress, failure and answer it shows. */
  operation: JobOperation<TResult>;
  /**
   * What a submit runs, or null while the fields name none. Memoised over exactly what the run reads: a new identity
   * means a field changed, which outdates whatever the previous run answered.
   */
  request: Nullable<TRequest>;
  /** Draws what a finished run answered. */
  renderResult: (result: TResult) => ReactNode;
  onSubmit: (request: TRequest) => void;
}

/**
 * A picker screen over one job: busy with its progress while it runs, then its failure or answer, a run rediscovered
 * after a window reload included. Submitting is offered only while there is a request to submit.
 */
export function JobPickerForm<TRequest, TResult>({
  operation,
  request,
  renderResult,
  isLoading = false,
  isSubmitDisabled = false,
  onSubmit,
  ...props
}: IJobPickerFormProps<TRequest, TResult>): ReactElement {
  const job: Nullable<IJobState> = operation.job;
  const result: Nullable<TResult> = operation.result;

  const onSubmitRequest = useCallback(() => {
    if (request !== null) {
      onSubmit(request);
    }
  }, [onSubmit, request]);

  useEffect(() => {
    operation.reset();
  }, [operation, request]);

  return (
    <PickerForm
      {...props}
      isLoading={isLoading || operation.isRunning}
      isSubmitDisabled={isSubmitDisabled || request === null}
      error={operation.error ?? undefined}
      status={job ? <JobProgressView job={job} onCancel={operation.cancel} /> : null}
      result={result === null ? null : renderResult(result)}
      onSubmit={onSubmitRequest}
    />
  );
}
