import { beforeEach, describe, expect, it } from "@jest/globals";
import { act } from "@testing-library/react";
import { EventBus } from "@wirestate/core";
import { runInAction } from "@wirestate/mobx";

import { SpriteEquipmentPackerApplication } from "@/applications/sprite-equipment-packer/SpriteEquipmentPackerApplication";
import { EJobKind, IJobSettledPayload, JOB_SETTLED_EVENT } from "@/core/jobs/lib";
import { JobsService } from "@/core/jobs/services/jobs";
import { IPackEquipmentResult, SpriteEquipmentService } from "@/core/sprite-equipment";
import { setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

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
  const { container } = mockInjectedService(SpriteEquipmentService);
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
    setMockInvokeResponses({});
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
