import { describe, expect, it } from "@jest/globals";

import { ArchiveProject } from "@/core/ipc/types/xrf-archive";
import { EPathEntryKind } from "@/core/path/entry-kind";
import { toDirectoryItemId, toFileItemId } from "@/core/ui/tree/path-tree";
import { mockArchiveFileDescriptor, mockArchivesProject } from "@/fixtures/mocks/archive.mocks";

import { listArchiveFiles } from "./files";
import { IArchiveTreeItem, isUnderArchiveDirectory, parseTree, toArchiveSelectionItemId } from "./tree";

const CONFIGS_DIALOGS: string = ["configs", "gameplay", "dialogs.xml"].join("\\");
const CONFIGS_BACKUP: string = ["configs_backup", "a.ltx"].join("\\");
const CONFIGS_DIRECTORY: string = ["configs", "gameplay", ""].join("\\");

describe("archive tree", () => {
  it("builds a directory-first tree with descriptors as leaf payloads", () => {
    const config = mockArchiveFileDescriptor({ name: "configs\\z.ltx" });
    const script = mockArchiveFileDescriptor({ name: "scripts\\actor.script" });
    const root = mockArchiveFileDescriptor({ name: "readme.ltx" });
    const items: Array<IArchiveTreeItem> = parseTree([root, script, config], "\\");

    expect(items.map((item) => item.label)).toEqual(["configs", "scripts", "readme.ltx"]);
    expect(items[0]).toMatchObject({ id: "directory:configs", kind: "directory" });

    const configs = items[0];

    expect(configs.kind).toBe("directory");

    if (configs.kind === EPathEntryKind.DIRECTORY) {
      expect(configs.children[0]).toMatchObject({
        id: "file:configs\\z.ltx",
        kind: "file",
        payload: config,
      });
    }
  });
});

describe("listArchiveFiles", () => {
  it("leaves out the directories a volume records", () => {
    // The name table lists them beside the files; taken as files they show as a leaf next to the directory of the
    // same name, and the volume above counts four entries for its two files.
    const project: ArchiveProject = mockArchivesProject([
      mockArchiveFileDescriptor({ name: "meshes", sizeCompressed: 0, sizeReal: 0 }),
      mockArchiveFileDescriptor({ name: "meshes\\actors\\", sizeCompressed: 0, sizeReal: 0 }),
      mockArchiveFileDescriptor({ name: "meshes\\actors\\stalker.ogf" }),
      mockArchiveFileDescriptor({ name: "readme.ltx" }),
    ]);

    expect(listArchiveFiles(project).map((descriptor) => descriptor.name)).toEqual([
      "meshes\\actors\\stalker.ogf",
      "readme.ltx",
    ]);
    expect(parseTree(listArchiveFiles(project), "\\").map((item) => item.id)).toEqual([
      "directory:meshes",
      "file:readme.ltx",
    ]);
  });

  it("has nothing to list without a project", () => {
    expect(listArchiveFiles(null)).toEqual([]);
  });
});

describe("isUnderArchiveDirectory", () => {
  function under(name: string, prefix: string, sizeReal: number = 128): boolean {
    return isUnderArchiveDirectory(mockArchiveFileDescriptor({ name, sizeReal }), prefix);
  }

  it("matches whole path segments only", () => {
    expect(under(CONFIGS_DIALOGS, "configs")).toBe(true);
    // A plain startsWith would pull the backup directory into an extraction of `configs`.
    expect(under(CONFIGS_BACKUP, "configs")).toBe(false);
  });

  it("takes everything for an empty prefix", () => {
    expect(under(CONFIGS_DIALOGS, "")).toBe(true);
  });

  it("ignores case and a trailing separator on the prefix", () => {
    expect(under(CONFIGS_DIALOGS, "CONFIGS")).toBe(true);
    expect(under(CONFIGS_DIALOGS, "configs\\")).toBe(true);
  });

  it("excludes what the backend will not write", () => {
    // Counting these would promise more files than extraction delivers.
    expect(under(CONFIGS_DIALOGS, "configs", 0)).toBe(false);
    expect(under(CONFIGS_DIRECTORY, "configs")).toBe(false);
  });
});

describe("toArchiveSelectionItemId", () => {
  it("addresses a selected file by the name its leaf was keyed on", () => {
    expect(
      toArchiveSelectionItemId({
        kind: EPathEntryKind.FILE,
        entry: mockArchiveFileDescriptor({ name: CONFIGS_DIALOGS }),
      })
    ).toBe(toFileItemId(CONFIGS_DIALOGS));
  });

  // A directory is a selection here as much as a file is, and answers a different kind of id.
  it("addresses a selected directory as a directory", () => {
    expect(toArchiveSelectionItemId({ kind: EPathEntryKind.DIRECTORY, path: "configs" })).toBe(
      toDirectoryItemId("configs")
    );
  });

  // The root is spelled as an empty prefix by the backend, and `toDirectoryItemId` turns that into the synthetic root.
  it("addresses the whole tree when the root directory is selected", () => {
    expect(toArchiveSelectionItemId({ kind: EPathEntryKind.DIRECTORY, path: "" })).toBe(toDirectoryItemId(""));
  });

  it("addresses nothing while nothing is selected", () => {
    expect(toArchiveSelectionItemId({ kind: "none" })).toBeNull();
  });
});
