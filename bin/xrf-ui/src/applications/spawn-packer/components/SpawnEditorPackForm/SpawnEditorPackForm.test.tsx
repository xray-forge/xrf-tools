import { beforeEach, describe, expect, it } from "@jest/globals";
import { fireEvent, waitFor } from "@testing-library/react";

import { SpawnConversionService } from "@/core/spawn/services/spawn-conversion.service";
import { setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

import { SpawnEditorPackForm } from "./SpawnEditorPackForm";

describe("SpawnEditorPackForm outcomes", () => {
  beforeEach(() => {
    window.localStorage.clear();
    window.localStorage.setItem("xrf.form.spawn-packer.source", "C:\\source");
    window.localStorage.setItem("xrf.form.spawn-packer.destination", "C:\\destination");
  });

  describe.each(["Source", "Output spawn"])("editing %s", (label) => {
    it.each(["success", "failure"])("clears the previous %s", async (outcome) => {
      setMockInvokeResponses({
        ["plugin:spawn|pack_file"]: () => {
          if (outcome === "failure") {
            throw new Error("Cannot process spawn");
          }

          return { operation: "pack", destination: "C:\\destination", outcome: "completed" };
        },
      });

      const view = renderWithProviders(<SpawnEditorPackForm />, {
        route: "/spawn-packer",
        bindings: [SpawnConversionService],
      });
      const submit = await view.findByRole("button", { name: "Pack" });

      await waitFor(() => expect(submit).toBeEnabled());
      fireEvent.click(submit);

      const message = outcome === "success" ? /Packed spawn/ : /Cannot process spawn/;

      expect(await view.findByText(message)).toBeInTheDocument();

      if (outcome === "success") {
        fireEvent.click(view.getByRole("button", { name: "Show parameters" }));
      }

      fireEvent.change(view.getByRole("textbox", { name: label }), { target: { value: "C:\\changed" } });

      expect(view.queryByText(message)).not.toBeInTheDocument();
    });
  });
});
