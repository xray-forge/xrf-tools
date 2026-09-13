import { beforeEach, describe, expect, it } from "@jest/globals";
import { act, fireEvent } from "@testing-library/react";
import { flowResult } from "@wirestate/mobx";

import { FALLBACK_PACK_CONFIG } from "@/applications/archives-packer/lib/pack-config";
import { PackerService } from "@/applications/archives-packer/services/packer";
import { ArchivePackResult } from "@/core/ipc/types/xrf-pack";
import { setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

import { ArchivesPackerApplication } from "./ArchivesPackerApplication";

const RESULT: ArchivePackResult = {
  outcome: "completed",
  volumes: ["C:\\out\\gamedata.db"],
  volumesOpened: ["C:\\out\\gamedata.db"],
  filesTotal: 1,
  filesSkipped: 0,
  filesStored: 1,
  filesCompressed: 0,
  filesAliased: 0,
  sizeSource: 100,
  sizeWritten: 120,
  duration: 1000,
  collectDuration: 100,
  writeDuration: 800,
  finalizeDuration: 100,
  speed: 100,
};

describe("Archives packer outcomes", () => {
  beforeEach(() => {
    window.localStorage.clear();
    window.localStorage.setItem("xrf.form.archives-packer.source", "C:\\in");
    window.localStorage.setItem("xrf.form.archives-packer.destination", "C:\\out");
  });

  describe.each(["Source", "Output"])("editing %s", (label) => {
    it.each(["success", "failure"])("clears the previous %s", async (outcome) => {
      setMockInvokeResponses({
        ["plugin:archives|default_pack_config"]: FALLBACK_PACK_CONFIG,
        ["plugin:archives|pack_directory"]: () => {
          if (outcome === "failure") {
            throw new Error("Cannot pack archives");
          }

          return RESULT;
        },
      });

      const { container, service } = mockInjectedService(PackerService);
      const view = renderWithProviders(<ArchivesPackerApplication />, { container, route: "/archives-packer" });

      await view.findByRole("textbox", { name: "Source" });
      await act(async () => {
        await flowResult(service.pack({ ...FALLBACK_PACK_CONFIG, source: "C:\\in", destination: "C:\\out" }, false));
      });

      const message = outcome === "success" ? "Last run" : "Cannot pack archives";

      expect(await view.findByText(message)).toBeInTheDocument();

      fireEvent.change(view.getByRole("textbox", { name: label }), { target: { value: "C:\\changed" } });

      expect(view.queryByText(message)).not.toBeInTheDocument();
      expect(service.isDirty).toBe(false);
    });
  });
});
