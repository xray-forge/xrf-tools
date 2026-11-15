import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { open, save } from "@tauri-apps/plugin-dialog";
import { exists } from "@tauri-apps/plugin-fs";

import { askToOpen, askToOpenDirectory, askToSave } from "@/core/path/native-dialog";

const mockOpen = jest.mocked(open<{ multiple: false }>);

describe("native dialogs", () => {
  beforeEach(() => {
    mockOpen.mockResolvedValue(null);
    jest.mocked(save).mockResolvedValue(null);
    jest.mocked(exists).mockResolvedValue(true);
  });

  it("answers the file picked, or null where the dialog was dismissed", async () => {
    mockOpen.mockResolvedValueOnce("C:\\game\\all.spawn");

    expect(await askToOpen({ title: "Open spawn" })).toBe("C:\\game\\all.spawn");
    expect(await askToOpen({ title: "Open spawn" })).toBeNull();
    expect(open).toHaveBeenCalledWith(expect.objectContaining({ directory: false, multiple: false }));
  });

  it("offers the filters it is given, and every file without them", async () => {
    await askToSave({ title: "Save", filters: [{ name: "ltx", extensions: ["ltx"] }] });
    await askToSave({ title: "Save" });

    expect(save).toHaveBeenNthCalledWith(
      1,
      expect.objectContaining({ filters: [{ name: "ltx", extensions: ["ltx"] }] })
    );
    expect(save).toHaveBeenNthCalledWith(2, expect.objectContaining({ filters: undefined }));
  });

  it("opens at the directory above a file that is no longer there", async () => {
    jest.mocked(exists).mockImplementation(async (path: unknown) => path === "C:\\projects\\gone");

    await askToOpen({ title: "Open", start: "C:\\projects\\gone\\all.spawn" });

    expect(open).toHaveBeenCalledWith(expect.objectContaining({ defaultPath: "C:\\projects\\gone" }));
  });

  it("leaves it to the host where the start leads nowhere", async () => {
    jest.mocked(exists).mockResolvedValue(false);

    await askToOpen({ title: "Open", start: "Q:\\nothing\\here" });
    await askToOpen({ title: "Open", start: null });

    expect(open).toHaveBeenNthCalledWith(1, expect.objectContaining({ defaultPath: undefined }));
    expect(open).toHaveBeenNthCalledWith(2, expect.objectContaining({ defaultPath: undefined }));
  });

  it("keeps a save's missing name whose directory is there, and a bare name as it is", async () => {
    jest.mocked(exists).mockImplementation(async (path: unknown) => path === "C:\\output");

    await askToSave({ title: "Save", start: "C:\\output\\packed.db" });
    await askToSave({ title: "Save", start: "extern.json" });

    expect(save).toHaveBeenNthCalledWith(1, expect.objectContaining({ defaultPath: "C:\\output\\packed.db" }));
    expect(save).toHaveBeenNthCalledWith(2, expect.objectContaining({ defaultPath: "extern.json" }));
    expect(exists).not.toHaveBeenCalledWith("extern.json");
  });

  it("offers a destination directory back whole, and an existing one's parent otherwise", async () => {
    jest.mocked(exists).mockImplementation(async (path: unknown) => path === "C:\\output");

    await askToOpenDirectory({ title: "Output", start: "C:\\output\\new", isDestination: true });
    await askToOpenDirectory({ title: "Source", start: "C:\\output\\new" });

    expect(open).toHaveBeenNthCalledWith(
      1,
      expect.objectContaining({ defaultPath: "C:\\output\\new", directory: true })
    );
    expect(open).toHaveBeenNthCalledWith(2, expect.objectContaining({ defaultPath: "C:\\output", directory: true }));
  });
});
