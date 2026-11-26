import { describe, expect, it, jest } from "@jest/globals";
import { ESystemCommand } from "@xrf/ipc/commands/system";
import { mockIpcCommands } from "@xrf/ipc/testing";
import { AppStorageUsage, EAppStorageKind } from "@xrf/ipc/types/xrf-app-system";
import { AppCacheTidy } from "@xrf/ipc/types/xrf-cache";

import { SettingsStorageService } from "@/core/settings-dialog/services/settings-storage";
import { mockInjectedService } from "@/fixtures/utils/container";
import { deferred } from "@/fixtures/utils/promise";

const TIDY: AppCacheTidy = { isDone: true, keptBytes: 100, keptEntries: 1, removedBytes: 300, removedEntries: 3 };

function mockUsage(bytes: number): AppStorageUsage {
  return {
    areas: [{ bytes, files: bytes ? 1 : 0, kind: EAppStorageKind.RENDERER_CACHE, lastUsed: null }],
    bytes,
    root: "C:\\xrf\\appdata",
  };
}

describe("SettingsStorageService", () => {
  it("measures the folder again once a kind is cleared, even where clearing failed", async () => {
    let kept: number = 400;

    mockIpcCommands({
      [ESystemCommand.GET_APP_STORAGE]: () => mockUsage(kept),
      [ESystemCommand.CLEAR_APP_STORAGE]: () => {
        kept = 0;

        throw new Error("in use");
      },
    });

    const { service } = mockInjectedService(SettingsStorageService);

    await service.show();

    expect(service.usage.value?.bytes).toBe(400);

    await service.clear(EAppStorageKind.RENDERER_CACHE);

    expect(service.usage.value?.bytes).toBe(0);
    expect(service.usage.error).toBeNull();
  });

  // Reopening the section measures again, which must not cancel the clear.
  it("finishes a clear that the section shown again overlaps", async () => {
    const cleared = deferred<null>();
    let kept: number = 400;

    mockIpcCommands({
      [ESystemCommand.GET_APP_STORAGE]: () => mockUsage(kept),
      [ESystemCommand.CLEAR_APP_STORAGE]: () =>
        cleared.promise.then(() => {
          kept = 0;

          return null;
        }),
    });

    const { service } = mockInjectedService(SettingsStorageService);
    // A flow method is declared as its generator; the caller is handed its promise.
    const clearing: unknown = service.clear(EAppStorageKind.RENDERER_CACHE);

    await service.show();

    expect(service.usage.value?.bytes).toBe(400);

    cleared.resolve(null);
    await clearing;

    expect(service.usage.value?.bytes).toBe(0);
  });

  it("says what the last tidy did until the section shows again", async () => {
    mockIpcCommands({
      [ESystemCommand.GET_APP_STORAGE]: () => mockUsage(100),
      [ESystemCommand.TIDY_APP_CACHE]: () => TIDY,
    });

    const { service } = mockInjectedService(SettingsStorageService);

    await service.tidyNow();

    expect(service.tidy.value).toEqual(TIDY);
    expect(service.usage.value?.bytes).toBe(100);

    await service.show();

    expect(service.tidy.value).toBeNull();
  });

  it("shows the folder it measured, and nothing before it has", async () => {
    const reveal = jest.fn((_: unknown) => null);

    mockIpcCommands({
      [ESystemCommand.GET_APP_STORAGE]: () => mockUsage(100),
      [ESystemCommand.REVEAL_PATH]: reveal,
    });

    const { service } = mockInjectedService(SettingsStorageService);

    await service.reveal();

    expect(reveal).not.toHaveBeenCalled();

    await service.show();
    await service.reveal();

    expect(reveal).toHaveBeenCalledWith({ path: "C:\\xrf\\appdata" });
  });
});
