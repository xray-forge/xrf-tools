import { describe, expect, it } from "@jest/globals";

import { IDialogTreeEntry, toDialogTreeEntries } from "@/applications/dialogs-editor/lib/dialog-tree";
import { DialogProjectDescriptor } from "@/core/bindings/types/xrf-dialog";

const SEPARATOR: string = "\\";

function project(
  files: Record<string, Array<string>>,
  dialogsPrefix: string = `configs${SEPARATOR}gameplay`
): DialogProjectDescriptor {
  return {
    mode: "gamedata",
    roots: { asset: null, roots: [] },
    dialogsPrefix,
    translationsPrefix: `configs${SEPARATOR}text`,
    isEditable: true,
    languages: [],
    textKeys: 0,
    files: Object.fromEntries(
      Object.entries(files).map(([logicalPath, ids]: [string, Array<string>]) => [
        logicalPath,
        {
          physicalPath: null,
          isEditable: false,
          encoding: "windows-1251",
          dialogs: ids.map((id: string) => ({ id, phrases: 1, priority: null })),
        },
      ])
    ),
    findings: [],
  };
}

describe("toDialogTreeEntries", () => {
  it("attaches nothing without a project", () => {
    expect(toDialogTreeEntries(null)).toEqual([]);
  });

  it("puts each dialog under the file declaring it, one level deep", () => {
    // The whole point of stripping the prefix: every project shares `configs\gameplay`, so keeping it
    // would cost two directory rows before the first file on every tree.
    const entries: Array<IDialogTreeEntry> = toDialogTreeEntries(
      project({ [`configs${SEPARATOR}gameplay${SEPARATOR}dialogs.xml`]: ["trader", "guide"] })
    );

    expect(entries.map((it: IDialogTreeEntry) => it.path)).toEqual([
      `dialogs.xml${SEPARATOR}trader`,
      `dialogs.xml${SEPARATOR}guide`,
    ]);
  });

  it("carries the file's own logical path on the leaf, not the relative one", () => {
    // The relative path is for display. A fetch addresses the file the project keyed it by.
    const [entry]: Array<IDialogTreeEntry> = toDialogTreeEntries(
      project({ [`configs${SEPARATOR}gameplay${SEPARATOR}dialogs.xml`]: ["trader"] })
    );

    expect(entry.payload).toEqual({
      id: "trader",
      logicalPath: `configs${SEPARATOR}gameplay${SEPARATOR}dialogs.xml`,
    });
  });

  it("keeps a nested file distinct from one sharing its name", () => {
    // Reducing to a bare file name would merge these two into one directory row holding both dialogs.
    const entries: Array<IDialogTreeEntry> = toDialogTreeEntries(
      project({
        [`configs${SEPARATOR}gameplay${SEPARATOR}dialogs.xml`]: ["a"],
        [`configs${SEPARATOR}gameplay${SEPARATOR}extra${SEPARATOR}dialogs.xml`]: ["b"],
      })
    );

    expect(entries.map((it: IDialogTreeEntry) => it.path)).toEqual([
      `dialogs.xml${SEPARATOR}a`,
      `extra${SEPARATOR}dialogs.xml${SEPARATOR}b`,
    ]);
  });

  it("leaves a file the prefix does not cover under its full path", () => {
    const entries: Array<IDialogTreeEntry> = toDialogTreeEntries(
      project({ [`mods${SEPARATOR}talks${SEPARATOR}dialogs.xml`]: ["a"] })
    );

    expect(entries[0].path).toBe(`mods${SEPARATOR}talks${SEPARATOR}dialogs.xml${SEPARATOR}a`);
  });

  it("strips a prefix the project spelled in another case", () => {
    // Logical paths are lower case by definition, but a caller may echo a user's typing into the
    // layout override that becomes this prefix.
    const entries: Array<IDialogTreeEntry> = toDialogTreeEntries(
      project({ [`configs${SEPARATOR}gameplay${SEPARATOR}dialogs.xml`]: ["a"] }, `CONFIGS${SEPARATOR}GAMEPLAY`)
    );

    expect(entries[0].path).toBe(`dialogs.xml${SEPARATOR}a`);
  });

  it("attaches at the root when the layout names no prefix", () => {
    const entries: Array<IDialogTreeEntry> = toDialogTreeEntries(project({ "dialogs.xml": ["a"] }, ""));

    expect(entries[0].path).toBe(`dialogs.xml${SEPARATOR}a`);
  });
});
