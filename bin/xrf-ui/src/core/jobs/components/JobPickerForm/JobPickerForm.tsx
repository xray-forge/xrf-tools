import { Nullable } from "@xrf/types";
import { ReactElement, ReactNode } from "react";

import { JobProgressView } from "@/core/jobs/components/JobProgressView";
import { IJobState } from "@/core/jobs/lib";
import { JobOperation } from "@/core/jobs/lib/job-operation";
import { IPickerFormProps, PickerForm } from "@/core/shell/editor/PickerForm";

export interface IJobPickerFormProps<T> extends Omit<IPickerFormProps, "error" | "status" | "result"> {
  /** The run the form starts, whose progress, failure and answer it shows. */
  operation: JobOperation<T>;
  /** Draws what a finished run answered. */
  renderResult: (result: T) => ReactNode;
}

/**
 * A picker screen over one job: busy while the job runs, its progress while it is found, its failure or its answer
 * once it settles, including a run rediscovered after the window reloaded.
 */
export function JobPickerForm<T>({
  operation,
  renderResult,
  isLoading = false,
  ...props
}: IJobPickerFormProps<T>): ReactElement {
  const job: Nullable<IJobState> = operation.job;
  const result: Nullable<T> = operation.result;

  return (
    <PickerForm
      {...props}
      isLoading={isLoading || operation.isRunning}
      error={operation.error ?? undefined}
      status={job ? <JobProgressView job={job} onCancel={operation.cancel} /> : null}
      result={result === null ? null : renderResult(result)}
    />
  );
}
