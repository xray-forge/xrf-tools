import { describe, expect, it } from "@jest/globals";
import { fireEvent, RenderResult } from "@testing-library/react";

import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import {
  ArchiveDescribeScope,
  ArchiveParticlesDescription,
  EArchiveDescribeScope,
  EArchiveReferenceStatus,
} from "@/core/ipc/types/xrf-app";
import {
  mockArchiveParticlesDescription,
  mockArchiveParticlesEffect,
  mockArchiveParticlesGroup,
  mockArchiveReference,
} from "@/fixtures/mocks/archive.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

import { ArchiveParticlesDescriptionView } from "./ArchiveParticlesDescriptionView";

const VOLUMES: ArchiveDescribeScope = { kind: EArchiveDescribeScope.VOLUMES, volumes: 3 };

function renderView(description: ArchiveParticlesDescription = mockArchiveParticlesDescription()): RenderResult {
  const { container } = mockInjectedService(ArchivesService);

  return renderWithProviders(<ArchiveParticlesDescriptionView description={description} scope={VOLUMES} />, {
    container,
  });
}

describe("ArchiveParticlesDescriptionView", () => {
  it("leads with what the library holds", () => {
    const { getByText } = renderView();

    expect(getByText("Effects (1)")).toBeTruthy();
    expect(getByText("Groups (1)")).toBeTruthy();
    expect(getByText("Every effect they name is defined here")).toBeTruthy();
  });

  it("names the texture an effect draws from and makes it selectable", () => {
    const { getByRole, getByText } = renderView();

    expect(getByRole("button", { name: "pfx\\pfx_smoke_a" })).toBeTruthy();
    expect(getByText(/drawn with particles\\add/)).toBeTruthy();
  });

  it("reports an action kind rather than its operands", () => {
    const { getByText } = renderView();

    expect(getByText("10 particles · 6 actions")).toBeTruthy();
    expect(getByText(/Source KillOld/)).toBeTruthy();
  });

  it("words an absent texture as the scope that was searched, without calling it a fault", () => {
    const { getByText, queryByText } = renderView(
      mockArchiveParticlesDescription({
        effects: [
          mockArchiveParticlesEffect({
            texture: mockArchiveReference({
              name: "pfx\\pfx_gone",
              path: "textures\\pfx\\pfx_gone.dds",
              entry: null,
              status: EArchiveReferenceStatus.ABSENT,
            }),
          }),
        ],
      })
    );

    expect(getByText(/Not in these 3 volumes/)).toBeTruthy();
    expect(queryByText(/missing/i)).toBeNull();
  });

  it("says when a group names an effect the library does not define", () => {
    const { getByText } = renderView(
      mockArchiveParticlesDescription({
        groups: [
          mockArchiveParticlesGroup({
            effects: [
              {
                effect: { name: "explosions\\gone", isDefined: false },
                onBirth: null,
                onPlay: { name: "explosions\\smoke", isDefined: true },
                onDead: null,
                from: 0,
                to: 1,
                flags: 6,
              },
            ],
          }),
        ],
      })
    );

    expect(getByText(/explosions\\gone \(not defined here\)/)).toBeTruthy();
    expect(getByText(/while playing explosions\\smoke/)).toBeTruthy();
  });

  it("filters both lists with one query", () => {
    const { getByLabelText, getByText } = renderView(
      mockArchiveParticlesDescription({
        effects: [mockArchiveParticlesEffect(), mockArchiveParticlesEffect({ name: "anomaly\\burn" })],
        groups: [mockArchiveParticlesGroup(), mockArchiveParticlesGroup({ name: "anomaly\\burn_group" })],
      })
    );

    fireEvent.change(getByLabelText("Filter effects and groups"), { target: { value: "anomaly" } });

    expect(getByText("Effects (1 of 2)")).toBeTruthy();
    expect(getByText("Groups (1 of 2)")).toBeTruthy();
  });
});
