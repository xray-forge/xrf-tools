import { describe, expect, it } from "@jest/globals";
import { Container } from "@wirestate/core";

import { KeybindCommandsService } from "@/core/commands";
import { CLEAR_LEVEL_SELECTION_KEYBIND_COMMAND, GO_TO_LEVEL_COORDINATES_KEYBIND_COMMAND } from "@/core/level/commands";
import { ILevelCamera } from "@/core/level/lib/camera/level-camera";
import { ELevelPick } from "@/core/level/lib/pick/level-pick";
import { LevelViewportService } from "@/core/level/services/level-viewport/level-viewport.service";
import { mockRenderLoadReport } from "@/core/render/fixtures/render.mocks";
import { mockContainer, mockInjectedService } from "@/fixtures/utils/container";

function camera(overrides: Partial<ILevelCamera> = {}): ILevelCamera {
  return { heading: 0, pitch: 0, position: { x: 0, y: 0, z: 0 }, ...overrides };
}

describe("LevelViewportService", () => {
  it("reports nothing about a camera that has never drawn", () => {
    const { service } = mockInjectedService(LevelViewportService);

    expect(service.camera).toBeNull();
    expect(service.load).toBeNull();
  });

  it("takes where the viewport's camera stands", () => {
    const { service } = mockInjectedService(LevelViewportService);

    service.noteCamera(camera({ heading: 1.5 }));

    expect(service.camera?.heading).toBe(1.5);
  });

  // `Escape` belongs to whatever has the keyboard: a panel's field or a popover keeps it, the scene clears with it.
  it("clears the selection on `Escape` only while the scene has the keyboard and something is selected", () => {
    // Provisioned rather than merely resolved: provisioning is what binds the handler and its guard to the buses.
    const container: Container = mockContainer([LevelViewportService]).provision();
    const commandsService: KeybindCommandsService = container.get(KeybindCommandsService);
    const service: LevelViewportService = container.get(LevelViewportService);

    service.noteSceneFocused(true);
    expect(commandsService.isAvailable(CLEAR_LEVEL_SELECTION_KEYBIND_COMMAND)).toBe(false);

    service.notePicked({
      isImpostor: false,
      kind: ELevelPick.SURFACE,
      mesh: null,
      place: null,
      point: { x: 0, y: 0, z: 0 },
      sector: 0,
      shaderId: 1,
    });
    expect(commandsService.isAvailable(CLEAR_LEVEL_SELECTION_KEYBIND_COMMAND)).toBe(true);

    service.noteSceneFocused(false);
    expect(commandsService.isAvailable(CLEAR_LEVEL_SELECTION_KEYBIND_COMMAND)).toBe(false);

    service.noteSceneFocused(true);
    service.clearPicked();
    expect(service.picked).toBeNull();
  });

  // The field starts at where the camera stands, so it opens only once the viewport has said, and a closed level shuts it.
  it("opens the go-to field once the camera has reported, and shuts it with the level", () => {
    const container: Container = mockContainer([LevelViewportService]).provision();
    const commandsService: KeybindCommandsService = container.get(KeybindCommandsService);
    const service: LevelViewportService = container.get(LevelViewportService);

    expect(commandsService.isAvailable(GO_TO_LEVEL_COORDINATES_KEYBIND_COMMAND)).toBe(false);

    service.noteCamera(camera());
    expect(commandsService.isAvailable(GO_TO_LEVEL_COORDINATES_KEYBIND_COMMAND)).toBe(true);

    expect(commandsService.execute(GO_TO_LEVEL_COORDINATES_KEYBIND_COMMAND)).toBe(true);
    expect(service.isGoingTo).toBe(true);

    service.clear();
    expect(service.isGoingTo).toBe(false);
  });

  // A closed viewer showing the last frame of the level before it is worse than showing nothing: it reads as a level
  // that is still open.
  it("forgets a closed level rather than keeping its last camera and load", () => {
    const { service } = mockInjectedService(LevelViewportService);

    service.noteCamera(camera());
    service.noteLoad(
      mockRenderLoadReport({ bytes: 1, isReady: true, sectors: 1, sectorsTotal: 1, textures: 0, texturesTotal: 0 })
    );
    service.clear();

    expect(service.camera).toBeNull();
    expect(service.load).toBeNull();
  });
});
