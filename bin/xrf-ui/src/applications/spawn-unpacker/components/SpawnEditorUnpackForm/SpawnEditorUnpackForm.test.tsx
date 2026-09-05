import { beforeEach, describe, expect, it } from "@jest/globals";
import { fireEvent, waitFor } from "@testing-library/react";

import { setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

import { SpawnEditorUnpackForm } from "./SpawnEditorUnpackForm";

describe("SpawnEditorUnpackForm outcomes", () => {
  beforeEach(() => {
    window.localStorage.clear();
    window.localStorage.setItem("xrf.form.spawn-unpacker.source", "C:\\source");
    window.localStorage.setItem("xrf.form.spawn-unpacker.destination", "C:\\destination");
  });

  describe.each(["Source", "Destination"])("editing %s", (label) => {
    it.each(["success", "failure"])("clears the previous %s", async (outcome) => {
      setMockInvokeResponses({
        ["plugin:spawn|unpack_file"]: () => {
          if (outcome === "failure") {
            throw new Error("Cannot process spawn");
          }

          return null;
        },
      });

      const view = renderWithProviders(<SpawnEditorUnpackForm />, { route: "/spawn-unpacker" });
      const submit = await view.findByRole("button", { name: "Unpack" });

      await waitFor(() => expect(submit).toBeEnabled());
      fireEvent.click(submit);

      const message = outcome === "success" ? /Successfully unpacked spawn/ : /Cannot process spawn/;

      expect(await view.findByText(message)).toBeInTheDocument();

      fireEvent.change(view.getByRole("textbox", { name: label }), { target: { value: "C:\\changed" } });

      expect(view.queryByText(message)).not.toBeInTheDocument();
    });
  });
});
