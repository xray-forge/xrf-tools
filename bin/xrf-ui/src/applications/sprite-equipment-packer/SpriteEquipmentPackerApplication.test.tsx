import { beforeEach, describe, expect, it } from "@jest/globals";
import { act, fireEvent } from "@testing-library/react";
import { EventBus } from "@wirestate/core";
import { runInAction } from "@wirestate/mobx";

import { EJobKind } from "@/core/ipc/types/xrf-app";
import { IJobSettledPayload, JOB_SETTLED_EVENT } from "@/core/jobs/lib";
import { JobsService } from "@/core/jobs/services/jobs";
import { IPackEquipmentResult } from "@/core/sprite-equipment/lib";
import { SpriteEquipmentPackerService } from "@/core/sprite-equipment/services/packer";
import { mockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockContainer, mockInjectedService } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

import { SPRITE_EQUIPMENT_PACKER_APPLICATION } from "./application";
import { SpriteEquipmentPackerApplication } from "./SpriteEquipmentPackerApplication";

const PACKED: IPackEquipmentResult = {
  outcome: "completed",
  duration: 1000,
  savedAt: "C:\\out\\equipment.dds",
  savedWidth: 1024,
  savedHeight: 512,
  packedCount: 12,
  skippedCount: 2,
};

function renderAdoptedPack() {
  const { container } = mockInjectedService(SpriteEquipmentPackerService);
  const jobs = container.get(JobsService);

  jobs.jobs = [
    {
      id: "pack-before-reload",
      kind: EJobKind.SPRITE_EQUIPMENT_PACK,
      request: null,
      progress: null,
      isAdopted: true,
      isCancelRequested: false,
    },
  ];

  const view = renderWithProviders(<SpriteEquipmentPackerApplication />, {
    container,
    route: "/sprite-equipment-packer",
  });

  function finish(result: IPackEquipmentResult | null, error: string | null = null): void {
    act(() => {
      runInAction(() => {
        jobs.jobs = [];
      });

      container.get(EventBus).emit<IJobSettledPayload>(JOB_SETTLED_EVENT, {
        id: "pack-before-reload",
        kind: EJobKind.SPRITE_EQUIPMENT_PACK,
        conclusion: error ? "failed" : "completed",
        result,
        error,
      });
    });
  }

  return { ...view, finish };
}

describe("Sprite equipment packer reload", () => {
  beforeEach(() => {
    window.localStorage.clear();
    window.localStorage.setItem("xrf.form.sprite-equipment-packer.source", "C:\\icons");
    window.localStorage.setItem("xrf.form.sprite-equipment-packer.output", "C:\\out\\equipment.dds");
    window.localStorage.setItem("xrf.form.sprite-equipment-packer.system-ltx", "C:\\configs\\system.ltx");
    setMockInvokeResponses({});
  });

  it("mounts and leaves the packer without restoring or closing an editor project", async () => {
    const runtime = await SPRITE_EQUIPMENT_PACKER_APPLICATION.load?.();
    const container = mockContainer([...(runtime?.container?.bindings ?? [])]);

    await container.provision();
    container.get(SpriteEquipmentPackerService);
    container.deprovision();
    container.unbindAll();

    expect(mockInvoke.mock.calls.filter(([command]) => command.startsWith("plugin:sprite-equipment|"))).toEqual([]);
  });

  describe.each(["Source", "Output", "System configuration", "DLTX"])("editing %s", (label) => {
    it.each(["success", "failure"])("clears an adopted %s", async (outcome) => {
      const view = renderAdoptedPack();

      await view.findByRole("button", { name: "Pack" });

      const message = outcome === "success" ? "12 file(s) packed" : "Cannot write equipment.dds";

      view.finish(outcome === "success" ? PACKED : null, outcome === "failure" ? message : null);

      expect(await view.findByText(message)).toBeInTheDocument();

      if (outcome === "success") {
        fireEvent.click(view.getByRole("button", { name: "Show parameters" }));
      }

      if (label === "DLTX") {
        fireEvent.click(view.getByRole("button", { name: label }));
      } else {
        fireEvent.change(view.getByRole("textbox", { name: label }), { target: { value: "C:\\changed" } });
      }

      expect(view.queryByText(message)).not.toBeInTheDocument();
    });
  });

  it("renders the result when the pack started before reload finishes", async () => {
    const { findByRole, findByText, finish } = renderAdoptedPack();

    expect(await findByRole("button", { name: "Pack" })).toBeDisabled();

    finish(PACKED);

    expect(await findByText("12 file(s) packed")).toBeInTheDocument();
    expect(await findByText("2 file(s) skipped")).toBeInTheDocument();
    expect(await findByText("1024x512 sprite")).toBeInTheDocument();
    expect(await findByRole("button", { name: "Back" })).toBeEnabled();
  });

  it("renders a failure from a pack started before reload", async () => {
    const { findByRole, findByText, finish } = renderAdoptedPack();

    await findByRole("button", { name: "Pack" });

    finish(null, "Cannot write equipment.dds");

    expect(await findByText("Cannot write equipment.dds")).toBeInTheDocument();
    expect(await findByRole("button", { name: "Back" })).toBeEnabled();
  });
});
