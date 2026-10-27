import { beforeEach, describe, expect, it } from "@jest/globals";
import { act, waitFor } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { EExportsCommand } from "@xrf/ipc/commands/exports";
import { mockInvoke, mockIpcCommands } from "@xrf/ipc/testing";
import { ExportSourceContent } from "@xrf/ipc/types/xrf-export";
import { Nullable } from "@xrf/types";

import { mockExportsProject } from "@/applications/exports-explorer/fixtures/exports.mocks";
import { ExportsService } from "@/applications/exports-explorer/services/exports";
import { mockSessionResponse, mockSessionSnapshot } from "@/fixtures/mocks/session.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

import { ExportSourceView } from "./ExportSourceView";

function mockSource(name: string, content: string, line: number = 18): ExportSourceContent {
  return { name, path: "effects/sound.ts", line, endLine: line + 2, content };
}

async function renderSource(name: string) {
  const container = mockContainer([ExportsService]);

  // Restored before the view reads, as the application shows the view only once a project is held.
  await container.get(ExportsService).onProvision();

  return renderWithProviders(<ExportSourceView name={name} />, { container });
}

describe("ExportSourceView", () => {
  beforeEach(() => {
    mockIpcCommands({
      [EExportsCommand.RESTORE_PROJECT]: mockSessionSnapshot(mockExportsProject()),
      [EExportsCommand.GET_SOURCE]: mockSource("xr_effects.play", 'extern("xr_effects.play", () => {});'),
    });
  });

  it("renders the body of the declaration it was asked for", async () => {
    const { findByLabelText } = await renderSource("xr_effects.play");

    expect(await findByLabelText("Source of xr_effects.play")).toHaveTextContent('extern("xr_effects.play", () => {})');
  });

  it("numbers lines from where the declaration starts in its file", async () => {
    mockIpcCommands({
      [EExportsCommand.RESTORE_PROJECT]: mockSessionSnapshot(mockExportsProject()),
      [EExportsCommand.GET_SOURCE]: mockSource("play", "line one\nline two\nline three", 18),
    });

    const { findByLabelText } = await renderSource("play");
    const gutter: Nullable<Element> = (await findByLabelText("Source of play")).querySelector("pre");

    // An excerpt is far more useful when its gutter still says where in the file it came from.
    expect(gutter?.textContent).toBe("18\n19\n20");
  });

  it("reports a failed read and retries the same declaration", async () => {
    mockIpcCommands({
      [EExportsCommand.RESTORE_PROJECT]: mockSessionSnapshot(mockExportsProject()),
      [EExportsCommand.GET_SOURCE]: () => {
        throw new Error("declaration file is gone");
      },
    });

    const { findByRole, getByRole, findByLabelText, queryByRole } = await renderSource("play");

    expect(await findByRole("alert")).toHaveTextContent("declaration file is gone");

    mockIpcCommands({
      [EExportsCommand.RESTORE_PROJECT]: mockSessionSnapshot(mockExportsProject()),
      [EExportsCommand.GET_SOURCE]: mockSource("play", "restored source"),
    });

    await userEvent.click(getByRole("button", { name: "Retry" }));

    expect(await findByLabelText("Source of play")).toHaveTextContent("restored source");
    expect(queryByRole("alert")).not.toBeInTheDocument();
    expect(mockInvoke.mock.calls.filter(([command]) => command === EExportsCommand.GET_SOURCE)).toEqual([
      [EExportsCommand.GET_SOURCE, { name: "play", sessionId: "fixture-session" }],
      [EExportsCommand.GET_SOURCE, { name: "play", sessionId: "fixture-session" }],
    ]);
  });

  it("reads the source again once the project is refreshed, which may have moved it", async () => {
    const container = mockContainer([ExportsService]);
    const service: ExportsService = container.get(ExportsService);

    mockIpcCommands({
      [EExportsCommand.RESTORE_PROJECT]: mockSessionSnapshot(mockExportsProject()),
      [EExportsCommand.GET_SOURCE]: mockSource("play", "first source"),
    });
    await service.onProvision();

    const { findByLabelText } = renderWithProviders(<ExportSourceView name={"play"} />, { container });

    expect(await findByLabelText("Source of play")).toHaveTextContent("first source");

    mockIpcCommands({
      [EExportsCommand.OPEN_PROJECT]: mockSessionResponse(() => mockExportsProject()),
      [EExportsCommand.CLOSE_PROJECT]: undefined,
      [EExportsCommand.GET_SOURCE]: mockSource("play", "edited source"),
    });

    await act(async () => {
      await service.refreshExportsProject();
    });

    expect(await findByLabelText("Source of play")).toHaveTextContent("edited source");
    expect(mockInvoke).toHaveBeenLastCalledWith(EExportsCommand.GET_SOURCE, {
      name: "play",
      sessionId: service.project.sessionId,
    });
  });

  it("ignores a read abandoned by a newer selection", async () => {
    // Clicking down a long list starts a read per declaration and they need not come back in order,
    // so the body of one already navigated away from must never replace what is on screen.
    const pending: Record<string, (value: ExportSourceContent) => void> = {};

    mockIpcCommands({
      [EExportsCommand.RESTORE_PROJECT]: mockSessionSnapshot(mockExportsProject()),
      [EExportsCommand.GET_SOURCE]: (parameters?: Record<string, unknown>) =>
        new Promise<ExportSourceContent>((resolve) => {
          pending[parameters?.name as string] = resolve;
        }),
    });

    const { rerender, findByLabelText } = await renderSource("first");

    rerender(
      <>
        <ExportSourceView name={"second"} />
      </>
    );

    pending.second(mockSource("second", "body of second"));

    // Asserted on the container rather than by text: colouring splits a body across spans.
    expect(await findByLabelText("Source of second")).toHaveTextContent("body of second");

    await act(async () => pending.first(mockSource("first", "body of first")));

    await waitFor(() => expect(findByLabelText("Source of second")).resolves.toBeInTheDocument());
    expect(await findByLabelText("Source of second")).not.toHaveTextContent("body of first");
  });
});
