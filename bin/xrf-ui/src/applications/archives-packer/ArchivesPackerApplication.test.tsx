import { beforeEach, describe, expect, it } from "@jest/globals";
import { act, fireEvent } from "@testing-library/react";
import { flowResult } from "@wirestate/mobx";

import { FALLBACK_PACK_CONFIG } from "@/applications/archives-packer/lib/pack-config";
import { PackerService } from "@/applications/archives-packer/services/packer";
import { mockArchivePackResult } from "@/fixtures/mocks/archive.mocks";
import { setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

import { ArchivesPackerApplication } from "./ArchivesPackerApplication";

describe("Archives packer outcomes", () => {
  beforeEach(() => {
    window.localStorage.clear();
    window.localStorage.setItem("xrf.form.archives-packer.source", "C:\\in");
    window.localStorage.setItem("xrf.form.archives-packer.destination", "C:\\out");
  });

  describe.each(["Source", "Output"])("editing %s", (label) => {
    it("clears the previous result", async () => {
      setMockInvokeResponses({
        ["plugin:archives|default_pack_config"]: FALLBACK_PACK_CONFIG,
        ["plugin:archives|pack_directory"]: mockArchivePackResult(),
      });

      const { container, service } = mockInjectedService(PackerService);
      const view = renderWithProviders(<ArchivesPackerApplication />, { container, route: "/archives-packer" });

      await view.findByRole("textbox", { name: "Source" });
      await act(async () => {
        await flowResult(service.pack({ ...FALLBACK_PACK_CONFIG, source: "C:\\in", destination: "C:\\out" }, false));
      });

      expect(await view.findByText("Last run")).toBeInTheDocument();

      fireEvent.change(view.getByRole("textbox", { name: label }), { target: { value: "C:\\changed" } });

      expect(view.queryByText("Last run")).not.toBeInTheDocument();
      expect(service.isDirty).toBe(false);
    });

    it("clears the previous failure", async () => {
      setMockInvokeResponses({
        ["plugin:archives|default_pack_config"]: FALLBACK_PACK_CONFIG,
        ["plugin:archives|pack_directory"]: () => {
          throw new Error("Cannot pack archives");
        },
      });

      const { container, service } = mockInjectedService(PackerService);
      const view = renderWithProviders(<ArchivesPackerApplication />, { container, route: "/archives-packer" });

      await view.findByRole("textbox", { name: "Source" });
      await act(async () => {
        await flowResult(service.pack({ ...FALLBACK_PACK_CONFIG, source: "C:\\in", destination: "C:\\out" }, false));
      });

      expect(await view.findByText("Cannot pack archives")).toBeInTheDocument();

      fireEvent.change(view.getByRole("textbox", { name: label }), { target: { value: "C:\\changed" } });

      expect(view.queryByText("Cannot pack archives")).not.toBeInTheDocument();
      expect(service.isDirty).toBe(false);
    });
  });
});
