import { describe, expect, it } from "@jest/globals";
import { RenderResult, within } from "@testing-library/react";
import { Container } from "@wirestate/core";

import { LevelSpawnModelFailure } from "@/core/ipc/types/xrf-app";
import { EXrayEngine } from "@/core/ipc/types/xrf-engine-target";
import { EMPTY_LEVEL_TEXTURE_REPORT } from "@/core/level/lib/texture/level-texture-report";
import { LevelLoadService, LevelViewportService } from "@/core/level/services";
import { setMockBulkResponses } from "@/fixtures/mocks/bulk.mocks";
import {
  mockLevelSpawnObject,
  mockLevelTextureReference,
  mockSectorDescription,
  mockSectorOutline,
  mockSelectedLevelDescription,
} from "@/fixtures/mocks/level.mocks";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { MockVisualBuffer } from "@/fixtures/mocks/visual.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

import { LevelProblemsPanel } from "./LevelProblemsPanel";

interface IRenderProblemsOptions {
  /** Whether the roots hold the level's one texture. */
  isPresent: boolean;
  /** A spawned visual the backend could not read, if any. */
  unreadable?: LevelSpawnModelFailure;
}

async function renderProblems({ isPresent, unreadable }: IRenderProblemsOptions): Promise<RenderResult> {
  const buffer: MockVisualBuffer = new MockVisualBuffer();
  const description = mockSectorDescription(buffer);
  const level = mockSelectedLevelDescription({
    sectors: [mockSectorOutline({ sector: 0 })],
    textures: [mockLevelTextureReference("stone", isPresent)],
  });

  setMockInvokeResponses({
    ["plugin:levels|open_level"]: mockSessionResponse(level),
    ["plugin:levels|open_sector"]: mockSessionResponse(description),
    // A spawn placing nothing drawn, which a clean level may well have.
    ["plugin:levels|open_spawn_objects"]: mockSessionResponse({ objects: [], visuals: [] }),
    ...(unreadable
      ? {
          ["plugin:levels|describe_spawn_models"]: mockSessionResponse({
            failures: [unreadable],
            hemi: [],
            models: [],
          }),
          ["plugin:levels|open_spawn_objects"]: mockSessionResponse({
            objects: [mockLevelSpawnObject()],
            visuals: [unreadable.name],
          }),
        }
      : {}),
  });

  setMockBulkResponses({
    "levels/read_sector": buffer.toArrayBuffer(),
  });

  const container: Container = mockContainer([LevelLoadService, LevelViewportService]);
  const service: LevelLoadService = container.get(LevelLoadService);

  await service.load({
    source: { kind: "asset", logicalPath: "levels\\zaton" },
    roots: level.roots,
    isDltx: false,
    engine: EXrayEngine.VANILLA,
  });
  await service.stream({ x: 0, y: 0, z: 0 });
  await service.whenHeldRead();

  // What the textures came to is the answer of whichever side uploaded them, so the panel is given it rather
  // than reaching for a set it can no longer see.
  container.get(LevelViewportService).noteTextures(
    isPresent
      ? EMPTY_LEVEL_TEXTURE_REPORT
      : {
          dressing: new Map(),
          problems: [{ reason: "Nothing in the mounted roots answers to 'stone'", reference: "stone" }],
          uploaded: 0,
        }
  );

  return renderWithProviders(<LevelProblemsPanel />, { container });
}

describe("LevelProblemsPanel", () => {
  it("says what was checked when a level read cleanly", async () => {
    const { getByTestId } = await renderProblems({ isPresent: true });

    expect(getByTestId("level-problems-panel").textContent).toContain("every shader table entry was described");
  });

  // The confusion the whole panel exists to end: a surface drew wrong and nothing anywhere named the file behind it,
  // so the level looked like the viewer was broken.
  it("names a texture the roots answer nothing for", async () => {
    const { getByTestId } = await renderProblems({ isPresent: false });
    const panel: HTMLElement = getByTestId("level-problems-panel");

    expect(panel.textContent).toContain("stone");
    expect(panel.textContent).toContain("Nothing in the mounted roots");
  });

  it("names a spawned visual the backend could not read, whose objects are absent", async () => {
    const { getByTestId } = await renderProblems({
      isPresent: true,
      unreadable: { name: "physics\\box", reason: "Failed to read visual" },
    });
    const panel: HTMLElement = getByTestId("level-problems-panel");

    expect(within(panel).getByTitle("physics\\box")).toHaveTextContent("box");
    expect(panel.textContent).toContain("Its objects are not drawn: Failed to read visual");
  });

  it("stands empty until a level is open", () => {
    const { getByTestId } = renderWithProviders(<LevelProblemsPanel />, {
      container: mockContainer([LevelLoadService, LevelViewportService]),
    });

    expect(getByTestId("level-problems-panel").textContent).toContain("No level open");
  });
});
