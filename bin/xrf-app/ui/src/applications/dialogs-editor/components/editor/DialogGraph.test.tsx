import { describe, expect, it, jest } from "@jest/globals";
import { Nullable } from "@xrf/types";

import { MOCK_DIALOG } from "@/applications/dialogs-editor/fixtures/dialogs.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

import { DialogGraph } from "./DialogGraph";

describe("DialogGraph", () => {
  it("starts with the inspected node selected, so the canvas does not report the inspection cleared", () => {
    const onSelect = jest.fn<(nodeId: Nullable<string>) => void>();

    renderWithProviders(<DialogGraph dialog={MOCK_DIALOG} selectedNodeId={"0"} onSelect={onSelect} />);

    expect(onSelect).not.toHaveBeenCalledWith(null);
    expect(onSelect).toHaveBeenLastCalledWith("0");
  });

  it("keeps the inspected node selected across a rebuild of the same dialog in another language", () => {
    const onSelect = jest.fn<(nodeId: Nullable<string>) => void>();
    const { rerender } = renderWithProviders(
      <DialogGraph dialog={MOCK_DIALOG} selectedNodeId={"0"} onSelect={onSelect} />
    );

    rerender(<DialogGraph dialog={{ ...MOCK_DIALOG, language: "rus" }} selectedNodeId={"0"} onSelect={onSelect} />);

    expect(onSelect).not.toHaveBeenCalledWith(null);
    expect(onSelect).toHaveBeenLastCalledWith("0");
  });
});
