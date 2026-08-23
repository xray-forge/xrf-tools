import { beforeEach, describe, expect, it } from "@jest/globals";
import { act, waitFor } from "@testing-library/react";

import { ExportSourceView } from "@/applications/exports-explorer/components/viewer/exports/ExportSourceView";
import { ExportsService } from "@/applications/exports-explorer/services/exports";
import { ExportSourceContent } from "@/core/bindings/types/xrf-export";
import { setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";
import { Nullable } from "@/lib/types/general";

function mockSource(name: string, content: string, line: number = 18): ExportSourceContent {
  return { name, path: "effects/sound.ts", line, endLine: line + 2, content };
}

function renderSource(name: string) {
  return renderWithProviders(<ExportSourceView name={name} />, { bindings: [ExportsService] });
}

describe("ExportSourceView", () => {
  beforeEach(() => {
    setMockInvokeResponses({
      ["plugin:exports|get_source"]: mockSource("xr_effects.play", 'extern("xr_effects.play", () => {});'),
    });
  });

  it("renders the body of the declaration it was asked for", async () => {
    const { findByLabelText } = renderSource("xr_effects.play");

    expect(await findByLabelText("Source of xr_effects.play")).toHaveTextContent('extern("xr_effects.play", () => {})');
  });

  it("numbers lines from where the declaration starts in its file", async () => {
    setMockInvokeResponses({
      ["plugin:exports|get_source"]: mockSource("play", "line one\nline two\nline three", 18),
    });

    const { findByLabelText } = renderSource("play");
    const gutter: Nullable<Element> = (await findByLabelText("Source of play")).querySelector("pre");

    // An excerpt is far more useful when its gutter still says where in the file it came from.
    expect(gutter?.textContent).toBe("18\n19\n20");
  });

  it("reports a read that failed", async () => {
    setMockInvokeResponses({
      ["plugin:exports|get_source"]: () => {
        throw new Error("declaration file is gone");
      },
    });

    const { findByText } = renderSource("play");

    expect(await findByText(/declaration file is gone/)).toBeInTheDocument();
  });

  it("ignores a read abandoned by a newer selection", async () => {
    // Clicking down a long list starts a read per declaration and they need not come back in order,
    // so the body of one already navigated away from must never replace what is on screen.
    const pending: Record<string, (value: ExportSourceContent) => void> = {};

    setMockInvokeResponses({
      ["plugin:exports|get_source"]: (parameters?: Record<string, unknown>) =>
        new Promise<ExportSourceContent>((resolve) => {
          pending[parameters?.name as string] = resolve;
        }),
    });

    const { rerender, findByLabelText } = renderSource("first");

    rerender(<ExportSourceView name={"second"} />);

    pending.second(mockSource("second", "body of second"));

    // Asserted on the container rather than by text: colouring splits a body across spans.
    expect(await findByLabelText("Source of second")).toHaveTextContent("body of second");

    await act(async () => pending.first(mockSource("first", "body of first")));

    await waitFor(() => expect(findByLabelText("Source of second")).resolves.toBeInTheDocument());
    expect(await findByLabelText("Source of second")).not.toHaveTextContent("body of first");
  });
});
