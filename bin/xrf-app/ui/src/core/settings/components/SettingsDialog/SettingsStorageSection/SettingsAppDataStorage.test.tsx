import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";
import { ESystemCommand } from "@xrf/ipc/commands/system";
import { mockIpcCommands, resetMockInvoke } from "@xrf/ipc/testing";
import { AppCacheTidy } from "@xrf/ipc/types/xrf-app-core";
import { AppStorageUsage, EAppStorageKind } from "@xrf/ipc/types/xrf-app-system";

import { SettingsAppDataStorage } from "@/core/settings/components/SettingsDialog/SettingsStorageSection/SettingsAppDataStorage";
import { DEFAULT_APP_CACHE_POLICY } from "@/core/settings/lib/app-cache";
import { describeLastUse } from "@/core/settings/lib/app-storage";
import { clearLocalStorage } from "@/core/storage";
import { renderWithProviders } from "@/fixtures/utils/render";

const DAY_MS: number = 24 * 60 * 60 * 1000;

/**
 * @param cached - Bytes of the renderer's cache, in as many files as there are hundreds of bytes.
 * @returns The application's folder holding them, a log, its preferences and a screenshot.
 */
function mockUsage(cached: number): AppStorageUsage {
  const now: number = Date.now();

  return {
    areas: [
      {
        bytes: cached,
        files: cached / 100,
        kind: EAppStorageKind.RENDERER_CACHE,
        lastUsed: cached ? now - DAY_MS * 3 : null,
      },
      { bytes: 0, files: 0, kind: EAppStorageKind.OTHER_CACHES, lastUsed: null },
      { bytes: 30, files: 1, kind: EAppStorageKind.LOGS, lastUsed: now },
      { bytes: 2, files: 1, kind: EAppStorageKind.PREFERENCES, lastUsed: now },
      { bytes: 50, files: 1, kind: EAppStorageKind.SAVED_FILES, lastUsed: now },
      { bytes: 0, files: 0, kind: EAppStorageKind.OTHER, lastUsed: null },
    ],
    bytes: cached + 82,
    root: "C:\\xrf\\appdata",
  };
}

describe("SettingsAppDataStorage", () => {
  beforeEach(() => {
    resetMockInvoke();
    clearLocalStorage();
  });

  it("lists what the application keeps on disk and clears the renderer's cache, measuring again", async () => {
    let kept: number = 400;
    const clear = jest.fn((_: unknown) => {
      kept = 0;

      return null;
    });

    mockIpcCommands({
      [ESystemCommand.GET_APP_STORAGE]: () => mockUsage(kept),
      [ESystemCommand.CLEAR_APP_STORAGE]: clear,
    });

    const { findByText, getAllByRole, getByRole, queryByRole } = renderWithProviders(<SettingsAppDataStorage />);

    expect(await findByText("Renderer cache")).toBeInTheDocument();
    expect(await findByText("4 files")).toBeInTheDocument();
    expect(await findByText("used 3 days ago")).toBeInTheDocument();
    expect(await findByText("Saved files")).toBeInTheDocument();
    // Only the renderer's cache is offered to clear; the saved files only to show.
    expect(getAllByRole("button", { name: "Clear" })).toHaveLength(1);
    expect(getByRole("button", { name: "Show" })).toBeInTheDocument();

    await userEvent.click(getByRole("button", { name: "Clear" }));
    await userEvent.click(getByRole("button", { name: "Clear" }));

    expect(clear).toHaveBeenCalledWith({ kind: EAppStorageKind.RENDERER_CACHE });
    // Measured again: the log, the preferences and the screenshot alone.
    expect(await findByText("82 B")).toBeInTheDocument();
    expect(queryByRole("button", { name: "Clear" })).not.toBeInTheDocument();
  });

  it("tidies the caches now by the limits set here, and says what it removed", async () => {
    const tidy = jest.fn((_: unknown): AppCacheTidy => ({
      isDone: true,
      keptBytes: 100,
      keptEntries: 1,
      removedBytes: 300,
      removedEntries: 3,
    }));

    mockIpcCommands({
      [ESystemCommand.GET_APP_STORAGE]: () => mockUsage(400),
      [ESystemCommand.TIDY_APP_CACHE]: tidy,
    });

    const { findByText, getByRole } = renderWithProviders(<SettingsAppDataStorage />);

    await userEvent.click(getByRole("button", { name: "Tidy now" }));

    expect(tidy).toHaveBeenCalledWith({ isForced: true, policy: DEFAULT_APP_CACHE_POLICY });
    expect(await findByText("Removed 3 unused, 300 B; kept 100 B.")).toBeInTheDocument();
  });

  it("says how long ago a kind was last used, by the day", () => {
    const now: number = Date.UTC(2026, 9, 30, 12);

    expect(describeLastUse(null, now)).toBe("");
    expect(describeLastUse(now - 1000, now)).toBe("used today");
    expect(describeLastUse(now - DAY_MS, now)).toBe("used yesterday");
    expect(describeLastUse(now - DAY_MS * 9, now)).toBe("used 9 days ago");
  });
});
