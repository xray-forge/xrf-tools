import { beforeEach, describe, expect, it } from "@jest/globals";
import { fireEvent, waitFor } from "@testing-library/react";

import { VerifierService } from "@/applications/configs-verifier/services/verifier";
import { LtxProjectVerifyResult } from "@/core/bindings/types/xrf-ltx";
import { setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

import { ConfigsVerifierApplication } from "./ConfigsVerifierApplication";

const RESULT: LtxProjectVerifyResult = {
  outcome: "completed",
  checkedFields: 10,
  checkedSections: 2,
  duration: 1000,
  startupDuration: 50,
  errors: [],
  invalidSections: 0,
  skippedSections: 0,
  totalFiles: 1,
  totalSections: 2,
  validSections: 2,
};

describe("Configs verifier outcomes", () => {
  beforeEach(() => {
    window.localStorage.clear();
    window.localStorage.setItem("xrf.form.configs-verifier.directory", "C:\\configs");
  });

  describe.each(["Configs directory", "DLTX"])("editing %s", (label) => {
    it.each(["success", "failure"])("clears the previous %s", async (outcome) => {
      setMockInvokeResponses({
        ["plugin:configs|verify_directory"]: () => {
          if (outcome === "failure") {
            throw new Error("Cannot read configs");
          }

          return RESULT;
        },
      });

      const { container } = mockInjectedService(VerifierService);
      const view = renderWithProviders(<ConfigsVerifierApplication />, { container, route: "/configs-verifier" });
      const submit = await view.findByRole("button", { name: "Verify" });

      await waitFor(() => expect(submit).toBeEnabled());
      fireEvent.click(submit);

      const message = outcome === "success" ? "All sections passed validation" : "Cannot read configs";

      expect(await view.findByText(message)).toBeInTheDocument();

      if (outcome === "success") {
        fireEvent.click(view.getByRole("button", { name: "Show parameters" }));
      }

      if (label === "DLTX") {
        fireEvent.click(view.getByRole("checkbox", { name: label }));
      } else {
        fireEvent.change(view.getByRole("textbox", { name: label }), { target: { value: "C:\\changed" } });
      }

      expect(view.queryByText(message)).not.toBeInTheDocument();
    });
  });
});
