import { describe, expect, it } from "@jest/globals";

import { ILevelCamera } from "@/core/level/lib/camera/level-camera";
import { LevelViewportService } from "@/core/level/services/level-viewport.service";
import { mockInjectedService } from "@/fixtures/utils/container";

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

  // A closed viewer showing the last frame of the level before it is worse than showing nothing: it reads as a level
  // that is still open.
  it("forgets a closed level rather than keeping its last camera and load", () => {
    const { service } = mockInjectedService(LevelViewportService);

    service.noteCamera(camera());
    service.noteLoad({ bytes: 1, isReady: true, sectors: 1, sectorsTotal: 1, textures: 0, texturesTotal: 0 });
    service.clear();

    expect(service.camera).toBeNull();
    expect(service.load).toBeNull();
  });
});
