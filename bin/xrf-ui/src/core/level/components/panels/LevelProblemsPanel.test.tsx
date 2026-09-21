import { describe, expect, it } from "@jest/globals";
import { RenderResult } from "@testing-library/react";
import { Container } from "@wirestate/core";

import { EMPTY_LEVEL_TEXTURE_REPORT } from "@/core/level/lib/texture/level-texture-report";
import { LevelLoadService, LevelViewportService } from "@/core/level/services";
import { mockDdsFile } from "@/fixtures/mocks/dds.mocks";
import {
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

async function renderProblems(isPresent: boolean): Promise<RenderResult> {
  const buffer: MockVisualBuffer = new MockVisualBuffer();
  const description = mockSectorDescription(buffer);
  const level = mockSelectedLevelDescription({
    sectors: [mockSectorOutline({ sector: 0 })],
    textures: [mockLevelTextureReference("stone", isPresent)],
  });

  setMockInvokeResponses({
    ["plugin:assets|read_asset"]: mockDdsFile(),
    ["plugin:levels|open_level"]: mockSessionResponse(level),
    ["plugin:levels|open_sector"]: mockSessionResponse(description),
    ["plugin:levels|read_sector"]: buffer.toArrayBuffer(),
  });

  const container: Container = mockContainer([LevelLoadService, LevelViewportService]);
  const service: LevelLoadService = container.get(LevelLoadService);

  await service.load({ kind: "asset", logicalPath: "levels\\zaton" }, level.roots);
  await service.stream({ x: 0, y: 0, z: 0 });

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
    const { getByTestId } = await renderProblems(true);

    expect(getByTestId("level-problems-panel").textContent).toContain("every shader table entry was described");
  });

  // The confusion the whole panel exists to end: a surface drew wrong and nothing anywhere named the file behind it,
  // so the level looked like the viewer was broken.
  it("names a texture the roots answer nothing for", async () => {
    const { getByTestId } = await renderProblems(false);
    const panel: HTMLElement = getByTestId("level-problems-panel");

    expect(panel.textContent).toContain("stone");
    expect(panel.textContent).toContain("Nothing in the mounted roots");
  });

  it("stands empty until a level is open", () => {
    const { getByTestId } = renderWithProviders(<LevelProblemsPanel />, {
      container: mockContainer([LevelLoadService, LevelViewportService]),
    });

    expect(getByTestId("level-problems-panel").textContent).toContain("No level open");
  });
});
