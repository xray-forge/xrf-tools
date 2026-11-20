import { describe, expect, it, jest } from "@jest/globals";
import { act, fireEvent } from "@testing-library/react";
import { Container, EventBus, Injectable } from "@wirestate/core";
import { ARCHIVES_JOB_UNPACK } from "@xrf/ipc/types/xrf-app-archives";
import { Logger } from "@xrf/logging";
import { Nullable } from "@xrf/types";
import { ReactElement } from "react";

import { IJobSettledPayload, JOB_SETTLED_EVENT } from "@/core/jobs/lib";
import { JobOperation } from "@/core/jobs/lib/job-operation";
import { JobPickerForm } from "@/core/shell/chrome/JobPickerForm";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

interface ITestRequest {
  path: string;
}

@Injectable()
class TestJobService {
  public readonly operation: JobOperation<string> = new JobOperation(new Logger(__MODULE_NAME__), {
    kinds: [ARCHIVES_JOB_UNPACK],
  });
}

function mockJobContainer(): { container: Container; service: TestJobService } {
  const container: Container = mockContainer([TestJobService]);

  return { container, service: container.get(TestJobService) };
}

function renderForm(
  container: Container,
  request: Nullable<ITestRequest>,
  onSubmit: (request: ITestRequest) => void = jest.fn()
): ReactElement {
  return (
    <JobPickerForm
      operation={container.get(TestJobService).operation}
      request={request}
      title={"Unpack"}
      submitLabel={"Run"}
      renderResult={(result: string) => <span>{result}</span>}
      onSubmit={onSubmit}
    />
  );
}

function settle(container: Container, result: string): void {
  act(() => {
    container.get(EventBus).emit<IJobSettledPayload>(JOB_SETTLED_EVENT, {
      id: "unpack",
      kind: ARCHIVES_JOB_UNPACK,
      conclusion: "completed",
      error: null,
      result,
    });
  });
}

describe("JobPickerForm", () => {
  it("offers no submit while the fields name no request", () => {
    const { container } = mockJobContainer();
    const view = renderWithProviders(renderForm(container, null), { route: "/archives-unpacker", container });

    expect(view.getByRole("button", { name: "Run" })).toBeDisabled();
  });

  it("submits the request the fields name", () => {
    const { container } = mockJobContainer();
    const request: ITestRequest = { path: "C:\\in" };
    const onSubmit = jest.fn();
    const view = renderWithProviders(renderForm(container, request, onSubmit), {
      route: "/archives-unpacker",
      container,
    });

    fireEvent.click(view.getByRole("button", { name: "Run" }));

    expect(onSubmit).toHaveBeenCalledWith(request);
  });

  it("keeps the answer while the request is the same one, and drops it once the request changes", () => {
    const { container, service } = mockJobContainer();
    const request: ITestRequest = { path: "C:\\in" };
    const view = renderWithProviders(renderForm(container, request), { route: "/archives-unpacker", container });

    settle(container, "unpacked");

    expect(view.getByText("unpacked")).toBeInTheDocument();

    view.rerender(renderForm(container, request));

    expect(view.getByText("unpacked")).toBeInTheDocument();

    view.rerender(renderForm(container, { path: "C:\\other" }));

    expect(view.queryByText("unpacked")).not.toBeInTheDocument();
    expect(service.operation.result).toBeNull();
  });
});
