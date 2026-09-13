import { describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import { renderWithProviders } from "@/fixtures/utils/render";

import { EditorFileHeader } from "./EditorFileHeader";

describe("EditorFileHeader", () => {
  it("names what is open and closes it", async () => {
    const onClose = jest.fn();
    const { getByRole, getByText } = renderWithProviders(
      <EditorFileHeader
        name={"configs/gameplay/dialogs.xml"}
        caption={"12.4 KB"}
        actions={<button type={"button"}>Extract</button>}
        onClose={onClose}
      />
    );

    expect(getByText("configs/gameplay/dialogs.xml")).toBeInTheDocument();
    expect(getByText("12.4 KB")).toBeInTheDocument();
    expect(getByRole("button", { name: "Extract" })).toBeInTheDocument();

    await userEvent.click(getByRole("button", { name: "Close file" }));

    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it("lets a surface name its subject something other than a file", () => {
    const { getByRole } = renderWithProviders(
      <EditorFileHeader
        name={"system.ltx"}
        closeLabel={"Close config"}
        closeDescription={"Clear the selection and close this config"}
        onClose={jest.fn()}
      />
    );

    const close: HTMLElement = getByRole("button", { name: "Close config" });

    expect(close).toHaveAccessibleDescription("Clear the selection and close this config");
  });
});
