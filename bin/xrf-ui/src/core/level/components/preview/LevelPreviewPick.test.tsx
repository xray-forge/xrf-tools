import { describe, expect, it } from "@jest/globals";
import { act, RenderResult } from "@testing-library/react";
import { Container } from "@wirestate/core";

import { LevelPreviewPick } from "@/core/level/components/preview/LevelPreviewPick";
import { ELevelPick, TLevelPick } from "@/core/level/lib/pick/level-pick";
import { LevelLoadService, LevelViewportService } from "@/core/level/services";
import { mockLevelSpawnObject } from "@/fixtures/mocks/level.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

function renderPicked(): { container: Container; view: RenderResult } {
  const container: Container = mockContainer([LevelLoadService, LevelViewportService]);

  return { container, view: renderWithProviders(<LevelPreviewPick />, { container }) };
}

function pick(container: Container, picked: TLevelPick): void {
  act(() => container.get(LevelViewportService).notePicked(picked));
}

describe("LevelPreviewPick", () => {
  it("names a picked surface's sector, entry and the place of the mesh it is one of", () => {
    const { container, view } = renderPicked();

    expect(view.queryByTestId("level-preview-pick")).not.toBeInTheDocument();

    pick(container, {
      isImpostor: false,
      kind: ELevelPick.SURFACE,
      mesh: 2,
      place: 17,
      point: { x: 1, y: 2, z: 3 },
      sector: 12,
      shaderId: 99,
    });

    const readout: HTMLElement = view.getByTestId("level-preview-pick");

    expect(readout).toHaveTextContent("sector 12 · entry 99 · mesh 2, place 17");
    expect(readout).toHaveTextContent("x 1.0 y 2.0 z 3.0");
  });

  it("names a picked object and the visual it stands as, and nothing once a click picks nothing", () => {
    const { container, view } = renderPicked();

    pick(container, {
      kind: ELevelPick.SPAWN,
      object: mockLevelSpawnObject({ name: "crate_2", section: "physic_object" }),
      point: { x: 0, y: 0, z: 0 },
      visual: "physics\\box",
    });

    expect(view.getByTestId("level-preview-pick")).toHaveTextContent("crate_2 · physic_object");
    expect(view.getByTestId("level-preview-pick")).toHaveTextContent("physics\\box");

    act(() => container.get(LevelViewportService).notePicked(null));

    expect(view.queryByTestId("level-preview-pick")).not.toBeInTheDocument();
  });
});
