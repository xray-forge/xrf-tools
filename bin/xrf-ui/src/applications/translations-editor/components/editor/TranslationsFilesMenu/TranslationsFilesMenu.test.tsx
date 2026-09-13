import { describe, expect, it, jest } from "@jest/globals";
import { fireEvent } from "@testing-library/react";

import { TranslationFile } from "@/core/ipc/types/xrf-translation";
import { renderWithProviders } from "@/fixtures/utils/render";

import { TranslationsFilesMenu } from "./TranslationsFilesMenu";

const FILES: Record<string, TranslationFile> = {
  "file_alpha.xml": { sources: {}, entries: { first: {} } },
  "file_beta.xml": { sources: {}, entries: { first: {}, second: {} } },
};

describe("TranslationsFilesMenu", () => {
  it("highlights the keyboard result independently of the open file and preserves dirty markers", () => {
    const onSelect = jest.fn();
    const view = renderWithProviders(
      <TranslationsFilesMenu
        files={FILES}
        dirtyFiles={["file_beta.xml"]}
        selected={"file_alpha.xml"}
        onSelect={onSelect}
      />
    );
    const input = view.getByRole("textbox", { name: "Filter translation files" });

    fireEvent.change(input, { target: { value: "file" } });
    fireEvent.keyDown(input, { key: "ArrowDown" });

    expect(view.getByText("file_beta.xml").closest("[role='button']")).toHaveClass("Mui-selected");
    expect(view.getByText("file_alpha.xml").closest("[role='button']")).not.toHaveClass("Mui-selected");
    expect(view.getByLabelText("Unsaved changes")).toBeInTheDocument();
    expect(view.getByText("2 entries")).toBeInTheDocument();

    fireEvent.keyDown(input, { key: "Enter" });

    expect(onSelect).toHaveBeenCalledWith("file_beta.xml");

    fireEvent.change(input, { target: { value: "" } });

    expect(view.getByText("file_alpha.xml").closest("[role='button']")).toHaveClass("Mui-selected");
  });
});
