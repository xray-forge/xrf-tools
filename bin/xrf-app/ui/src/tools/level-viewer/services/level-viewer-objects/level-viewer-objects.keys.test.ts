import { beforeEach, describe, expect, it } from "@jest/globals";
import { Container } from "@wirestate/core";
import { ELevelsCommand } from "@xrf/ipc/commands/levels";
import { mockIpcCommands, resetMockInvoke } from "@xrf/ipc/testing";
import { LevelObject } from "@xrf/ipc/types/xrf-app-levels";

import { KeybindCommandsService } from "@/core/commands";
import {
  CLEAR_LEVEL_SELECTION_KEYBIND_COMMAND,
  GO_TO_LEVEL_COORDINATES_KEYBIND_COMMAND,
  HIDE_LEVEL_SELECTION_KEYBIND_COMMAND,
  REVEAL_LEVEL_OBJECTS_KEYBIND_COMMAND,
  TOGGLE_LEVEL_HELPERS_KEYBIND_COMMAND,
} from "@/core/level/commands";
import { mockLevelViewContainer } from "@/core/level/fixtures/level-container";
import { mockLevelObject } from "@/core/level/fixtures/level.mocks";
import { ELevelPick } from "@/core/level/lib/pick/level-pick";
import { LevelViewService } from "@/core/level/services/level-view";
import { LevelViewportService } from "@/core/level/services/level-viewport";
import { clearLocalStorage } from "@/core/storage";
import { mockContainer } from "@/fixtures/utils/container";
import { LevelViewerObjectsService } from "@/tools/level-viewer/services/level-viewer-objects";
import { LevelViewerSnapshotService } from "@/tools/level-viewer/services/level-viewer-snapshot";

interface IViewerKeys {
  commandsService: KeybindCommandsService;
  viewportService: LevelViewportService;
  viewService: LevelViewService;
}

// Provisioned rather than merely resolved: provisioning is what binds the handlers and their guards to the buses.
function provisionViewer(): IViewerKeys {
  const { container } = mockLevelViewContainer([LevelViewerObjectsService, LevelViewerSnapshotService]);

  container.provision();

  return {
    commandsService: container.get(KeybindCommandsService),
    viewportService: container.get(LevelViewportService),
    viewService: container.get(LevelViewService),
  };
}

describe("the level viewer's keys", () => {
  beforeEach(() => {
    clearLocalStorage();
    resetMockInvoke();
    mockIpcCommands({ [ELevelsCommand.RESTORE_LEVEL]: null });
  });

  // `Escape` belongs to whatever has the keyboard: a panel's field or a popover keeps it, the scene clears with it.
  it("clears the selection on `Escape` only while the scene has the keyboard and something is selected", () => {
    const { commandsService, viewportService } = provisionViewer();

    viewportService.noteSceneFocused(true);
    expect(commandsService.isAvailable(CLEAR_LEVEL_SELECTION_KEYBIND_COMMAND)).toBe(false);

    viewportService.notePicked({
      isImpostor: false,
      kind: ELevelPick.SURFACE,
      mesh: null,
      place: null,
      point: { x: 0, y: 0, z: 0 },
      sector: 0,
      shaderId: 1,
    });
    expect(commandsService.isAvailable(CLEAR_LEVEL_SELECTION_KEYBIND_COMMAND)).toBe(true);

    viewportService.noteSceneFocused(false);
    expect(commandsService.isAvailable(CLEAR_LEVEL_SELECTION_KEYBIND_COMMAND)).toBe(false);

    viewportService.noteSceneFocused(true);
    expect(commandsService.execute(CLEAR_LEVEL_SELECTION_KEYBIND_COMMAND)).toBe(true);
    expect(viewportService.picked).toBeNull();
  });

  it("hides the selection and reveals what is hidden", () => {
    const { commandsService, viewportService } = provisionViewer();
    const object: LevelObject = mockLevelObject();

    expect(commandsService.isAvailable(HIDE_LEVEL_SELECTION_KEYBIND_COMMAND)).toBe(false);
    expect(commandsService.isAvailable(REVEAL_LEVEL_OBJECTS_KEYBIND_COMMAND)).toBe(false);

    viewportService.noteSelected([object], {
      kind: ELevelPick.OBJECT,
      object,
      point: { x: 0, y: 0, z: 0 },
      visual: "",
    });
    expect(commandsService.execute(HIDE_LEVEL_SELECTION_KEYBIND_COMMAND)).toBe(true);
    expect(viewportService.hidden).toEqual([object]);

    expect(commandsService.execute(REVEAL_LEVEL_OBJECTS_KEYBIND_COMMAND)).toBe(true);
    expect(viewportService.hidden).toEqual([]);
  });

  it("toggles the helpers", () => {
    const { commandsService, viewService } = provisionViewer();
    const isShown: boolean = viewService.helpers.isShown;

    expect(commandsService.execute(TOGGLE_LEVEL_HELPERS_KEYBIND_COMMAND)).toBe(true);
    expect(viewService.helpers.isShown).toBe(!isShown);
  });

  // The field starts where the camera stands, so it waits for the viewport's report.
  it("opens the go-to field once the camera has reported", () => {
    const { commandsService, viewportService } = provisionViewer();

    expect(commandsService.isAvailable(GO_TO_LEVEL_COORDINATES_KEYBIND_COMMAND)).toBe(false);

    viewportService.noteCamera({ heading: 0, pitch: 0, position: { x: 0, y: 0, z: 0 } });
    expect(commandsService.execute(GO_TO_LEVEL_COORDINATES_KEYBIND_COMMAND)).toBe(true);
    expect(viewportService.isGoingTo).toBe(true);
  });
});

// The level editor provides both core services for its picture and selects through its own service.
describe("a tool providing the core level services alone", () => {
  it("takes none of the viewer's keys", () => {
    const container: Container = mockContainer([LevelViewService, LevelViewportService]).provision();
    const commandsService: KeybindCommandsService = container.get(KeybindCommandsService);
    const viewportService: LevelViewportService = container.get(LevelViewportService);

    viewportService.noteCamera({ heading: 0, pitch: 0, position: { x: 0, y: 0, z: 0 } });

    for (const command of [
      GO_TO_LEVEL_COORDINATES_KEYBIND_COMMAND,
      HIDE_LEVEL_SELECTION_KEYBIND_COMMAND,
      REVEAL_LEVEL_OBJECTS_KEYBIND_COMMAND,
      TOGGLE_LEVEL_HELPERS_KEYBIND_COMMAND,
    ]) {
      expect(commandsService.execute(command)).toBe(false);
    }
  });
});
