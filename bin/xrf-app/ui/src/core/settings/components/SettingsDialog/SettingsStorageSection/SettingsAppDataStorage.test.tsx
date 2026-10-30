import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";
import { ESystemCommand } from "@xrf/ipc/commands/system";
import { mockIpcCommands, resetMockInvoke } from "@xrf/ipc/testing";
import { AppStorageUsage, EAppStorageKind } from "@xrf/ipc/types/xrf-app-system";

import { SettingsAppDataStorage } from "@/core/settings/components/SettingsDialog/SettingsStorageSection/SettingsAppDataStorage";
import { renderWithProviders } from "@/fixtures/utils/render";

/**
 * @param cached - Bytes of the renderer's cache, in as many files as there are hundreds of bytes.
 * @returns The application's folder holding them, a log and its preferences.
 */
function mockUsage(cached: number): AppStorageUsage {
  return {
    areas: [
      { bytes: cached, files: cached / 100, kind: EAppStorageKind.RENDERER_CACHE },
      { bytes: 0, files: 0, kind: EAppStorageKind.OTHER_CACHES },
      { bytes: 30, files: 1, kind: EAppStorageKind.LOGS },
      { bytes: 2, files: 1, kind: EAppStorageKind.PREFERENCES },
      { bytes: 0, files: 0, kind: EAppStorageKind.OTHER },
    ],
    bytes: cached + 32,
    root: "C:\\xrf\\appdata",
  };
}

describe("SettingsAppDataStorage", () => {
  beforeEach(() => {
    resetMockInvoke();
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
    // Only the renderer's cache is offered to clear.
    expect(getAllByRole("button", { name: "Clear" })).toHaveLength(1);

    await userEvent.click(getByRole("button", { name: "Clear" }));
    await userEvent.click(getByRole("button", { name: "Clear" }));

    expect(clear).toHaveBeenCalledWith({ kind: EAppStorageKind.RENDERER_CACHE });
    // Measured again: the log and the preferences alone.
    expect(await findByText("32 B")).toBeInTheDocument();
    expect(queryByRole("button", { name: "Clear" })).not.toBeInTheDocument();
  });
});
