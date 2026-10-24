import { describe, expect, it } from "@jest/globals";
import { act, RenderResult, waitFor } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { ReactElement } from "react";

import { useEditorBusy, useEditorDirty, useWindowCloseGuard } from "@/core/shell/editor-lifecycle";
import {
  mockAppWindow,
  mockInvoke,
  requestMockWindowClose,
  setMockInvokeResponses,
} from "@/fixtures/mocks/tauri.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

const APPROVE: string = "plugin:system|approve_window_close";

function Window({ dirtyCount = 0, isBusy = false }: { dirtyCount?: number; isBusy?: boolean }): ReactElement {
  useEditorDirty(dirtyCount);
  useEditorBusy(isBusy);
  useWindowCloseGuard();

  return <div>window</div>;
}

async function renderWindow(content: ReactElement): Promise<RenderResult> {
  const result: RenderResult = renderWithProviders(content);

  // The guard subscribes asynchronously, as Tauri's listener does.
  await waitFor(() => expect(mockAppWindow.onCloseRequested).toHaveBeenCalled());

  return result;
}

async function requestClose(): Promise<boolean> {
  let isHeld: boolean = false;

  await act(async () => {
    isHeld = await requestMockWindowClose();
  });

  return isHeld;
}

describe("useWindowCloseGuard", () => {
  it("holds every close for the page, then approves it and asks again when nothing is at stake", async () => {
    await renderWindow(<Window />);

    expect(await requestClose()).toBe(true);

    await waitFor(() => expect(mockAppWindow.close).toHaveBeenCalledTimes(1));
    expect(mockInvoke).toHaveBeenCalledWith(APPROVE);

    // Asked again, the close must go through: the page stopped listening before asking.
    expect(await requestClose()).toBe(false);
  });

  it("stays guarded when the backend refuses the approval, so the next close asks again", async () => {
    setMockInvokeResponses({
      [APPROVE]: () => {
        throw new Error("refused");
      },
    });

    await renderWindow(<Window />);

    expect(await requestClose()).toBe(true);
    await waitFor(() => expect(mockInvoke).toHaveBeenCalledWith(APPROVE));

    expect(mockAppWindow.close).not.toHaveBeenCalled();
    expect(await requestClose()).toBe(true);
  });

  it("guards the window again when the approved close fails, so the next close asks again", async () => {
    mockAppWindow.close.mockRejectedValueOnce(new Error("busy"));

    await renderWindow(<Window />);

    expect(await requestClose()).toBe(true);
    await waitFor(() => expect(mockAppWindow.onCloseRequested).toHaveBeenCalledTimes(2));

    expect(await requestClose()).toBe(true);
  });

  it("asks about unsaved work before the window goes", async () => {
    const { getByRole, getByText } = await renderWindow(<Window dirtyCount={1} />);

    await requestClose();

    expect(getByText("Leave without saving?")).toBeInTheDocument();
    expect(mockAppWindow.close).not.toHaveBeenCalled();

    await userEvent.click(getByRole("button", { name: "Discard and leave" }));

    await waitFor(() => expect(mockAppWindow.close).toHaveBeenCalledTimes(1));
  });

  it("asks rather than refusing while work runs, so the window can always be closed", async () => {
    const { findByText, getByRole } = await renderWindow(<Window isBusy />);

    await requestClose();
    await userEvent.click(getByRole("button", { name: "Stay" }));

    expect(mockAppWindow.close).not.toHaveBeenCalled();

    await requestClose();

    expect(await findByText("Exit while working?")).toBeInTheDocument();

    await userEvent.click(getByRole("button", { name: "Exit anyway" }));

    await waitFor(() => expect(mockAppWindow.close).toHaveBeenCalledTimes(1));
  });
});
