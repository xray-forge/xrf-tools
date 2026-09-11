import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { open, save } from "@tauri-apps/plugin-dialog";
import { act, renderHook } from "@testing-library/react";

import { type IPathStateOptions, type TPathState, usePathState } from "@/core/ui/form/file-picker/use-path-state";
import { mockIsTauri } from "@/fixtures/mocks/tauri.mocks";
import { Nullable } from "@/lib/types/general";

const mockOpen = jest.mocked(open<{ multiple: false }>);

describe("usePathState", () => {
  // Awaited because the hook asks where to open before it opens: the dialog call is no longer reached synchronously.
  async function select(options: IPathStateOptions): Promise<void> {
    const { result } = renderHook(() => usePathState(options));

    await act(() => (result.current as TPathState)[2]());
  }

  beforeEach(() => {
    mockOpen.mockResolvedValue(null);
    jest.mocked(save).mockResolvedValue(null);
  });

  it("asks for a file to read by default", async () => {
    await select({ title: "Pick a file" });

    expect(open).toHaveBeenCalledWith(expect.objectContaining({ directory: false }));
    expect(save).not.toHaveBeenCalled();
  });

  it("asks for a directory when one is wanted", async () => {
    await select({ title: "Pick a directory", isDirectory: true });

    expect(open).toHaveBeenCalledWith(expect.objectContaining({ directory: true }));
    expect(save).not.toHaveBeenCalled();
  });

  it("asks where to save a file when the destination is a file that need not exist", async () => {
    await select({ title: "Pick an output sprite", isSave: true });

    expect(save).toHaveBeenCalled();
    expect(open).not.toHaveBeenCalled();
  });

  /**
   * The combination three screens actually use, and the one that was wrong.
   *
   * @remarks
   * An output directory is both things at once: it names a directory, and it need not exist yet.
   * Branching on `isSave` first put a file-name dialog in front of every screen whose destination is
   * a folder - the archives packer, the archives unpacker, and the translations parser. `isSave` says
   * whether the path must already exist, which is a question for validation, not for which dialog to
   * open; `isDirectory` says what kind of thing is being picked, and a save dialog cannot pick one.
   */
  it("still asks for a directory when the destination is a directory that need not exist", async () => {
    await select({ title: "Select output directory", isDirectory: true, isSave: true });

    expect(open).toHaveBeenCalledWith(expect.objectContaining({ directory: true }));
    expect(save).not.toHaveBeenCalled();
  });

  it("opens nothing while disabled", async () => {
    await select({ title: "Pick a directory", isDirectory: true, isDisabled: true });

    expect(open).not.toHaveBeenCalled();
    expect(save).not.toHaveBeenCalled();
  });

  it("opens no native dialog outside Tauri", async () => {
    mockIsTauri.mockReturnValue(false);

    const { result } = renderHook(() => usePathState({ title: "Pick a file" }));

    const selected: Nullable<string> = await act(() => (result.current as TPathState)[2]());

    expect(selected).toBeNull();
    expect(open).not.toHaveBeenCalled();
    expect(save).not.toHaveBeenCalled();
  });

  it("opens no native save dialog outside Tauri", async () => {
    mockIsTauri.mockReturnValue(false);

    const { result } = renderHook(() => usePathState({ isSave: true, title: "Save a file" }));

    const selected: Nullable<string> = await act(() => (result.current as TPathState)[2]());

    expect(selected).toBeNull();
    expect(open).not.toHaveBeenCalled();
    expect(save).not.toHaveBeenCalled();
  });
});
