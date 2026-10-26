import { describe, expect, it } from "@jest/globals";
import { RenderResult, waitFor, within } from "@testing-library/react";
import { Container } from "@wirestate/core";
import { AsyncState } from "@xrf/async";
import { ELevelsCommand } from "@xrf/ipc/commands/levels";
import { mockIpcCommands } from "@xrf/ipc/testing";
import { EXrayEngineChoice } from "@xrf/ipc/types/xrf-engine-target";
import { ERenderTextureGroup, RenderLoadFailure } from "@xrf/ipc/types/xrf-renderer";

import { EMPTY_LEVEL_TEXTURE_REPORT } from "@/core/level/lib/texture/level-texture-report";
import { LevelLoadService } from "@/core/level/services/level-load.service";
import { LevelViewportService } from "@/core/level/services/level-viewport.service";
import {
  mockLevelPartResponses,
  mockLevelSpawnObject,
  mockLevelTextureReference,
  mockSectorOutline,
  mockSelectedLevelDescription,
} from "@/fixtures/mocks/level.mocks";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

import { LevelProblemsPanel } from "./LevelProblemsPanel";

interface IRenderProblemsOptions {
  /** Whether the roots hold the level's one texture. */
  isPresent: boolean;
  /** A spawned visual the renderer could not read, if any. */
  unreadable?: RenderLoadFailure;
  /** How asking the renderer about the resident level went: answered unless said. */
  description?: AsyncState<void>;
}

async function renderProblems({
  isPresent,
  unreadable,
  description = AsyncState.ready<void>(null),
}: IRenderProblemsOptions): Promise<RenderResult> {
  const level = mockSelectedLevelDescription({ sectors: [mockSectorOutline({ sector: 0 })] });

  mockIpcCommands({
    [ELevelsCommand.OPEN_LEVEL]: mockSessionResponse(level),
    ...mockLevelPartResponses({ textures: [mockLevelTextureReference("stone", isPresent)] }),
    // A spawn placing one object, or nothing drawn, which a clean level may well have.
    [ELevelsCommand.DESCRIBE_SPAWN_OBJECTS]: mockSessionResponse(
      unreadable ? { objects: [mockLevelSpawnObject()], visuals: [unreadable.name] } : { objects: [], visuals: [] }
    ),
  });

  const container: Container = mockContainer([LevelLoadService, LevelViewportService]);
  const service: LevelLoadService = container.get(LevelLoadService);
  const viewport: LevelViewportService = container.get(LevelViewportService);

  await service.load({
    source: { kind: "asset", logicalPath: "levels\\zaton" },
    roots: level.roots,
    isDltx: false,
    engine: EXrayEngineChoice.VANILLA,
  });
  // The spawn is listed after the level opens; waited for, so the panel renders once with both.
  await waitFor(() => expect(service.spawnReport.isListed).toBe(true));

  // What the textures came to and what the level could not draw are the renderer's answers, so the panel is given
  // them rather than reaching for what it can no longer see.
  viewport.noteTextures(
    isPresent
      ? EMPTY_LEVEL_TEXTURE_REPORT
      : {
          deferred: 0,
          dressing: new Map(),
          problems: [
            {
              group: ERenderTextureGroup.SURFACES,
              reason: "Nothing in the mounted roots answers to 'stone'",
              reference: "stone",
            },
          ],
          uploaded: 0,
        }
  );
  viewport.noteProblems({ models: unreadable ? [unreadable] : [], sectors: [], skipped: [] });
  viewport.noteDescription(description);

  return renderWithProviders(<LevelProblemsPanel />, { container });
}

describe("LevelProblemsPanel", () => {
  it("says what was checked when a level read cleanly", async () => {
    const { getByTestId } = await renderProblems({ isPresent: true });

    expect(getByTestId("level-problems-panel").textContent).toContain(
      "its textures, shader entries, sectors' drawables and spawned visuals"
    );
  });

  // Before the level is resident the renderer has said nothing, which is not the same as having found nothing.
  it("says the renderer's part is not read yet rather than calling the level clean", async () => {
    const { getByTestId } = await renderProblems({ description: AsyncState.idle(), isPresent: true });
    const panel: HTMLElement = getByTestId("level-problems-panel");

    expect(getByTestId("editor-problems-status")).toHaveTextContent(
      "Not read yet: the renderer says what it could not draw once the level is resident."
    );
    expect(panel.textContent).not.toContain("No problems found");
  });

  it("says why the renderer's part could not be read", async () => {
    const { getByTestId } = await renderProblems({
      description: AsyncState.failed(new Error("The viewport is gone")),
      isPresent: true,
    });

    expect(getByTestId("editor-problems-status")).toHaveTextContent(
      "Could not read what it could not draw: The viewport is gone."
    );
    expect(getByTestId("level-problems-panel").textContent).not.toContain("No problems found");
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
