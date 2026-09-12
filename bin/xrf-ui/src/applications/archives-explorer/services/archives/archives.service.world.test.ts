import { describe, expect, it } from "@jest/globals";

import { ArchivesService } from "@/applications/archives-explorer/services/archives/index";
import { EArchiveSubject } from "@/core/archive";
import { ArchiveWorldEntry } from "@/core/bindings/types/xrf-app";
import { ArchiveReadResult } from "@/core/bindings/types/xrf-archive";
import { XrayRoots } from "@/core/bindings/types/xrf-vfs";
import {
  mockArchivedContainer,
  mockArchivesVolumes,
  mockArchivesWorldSubject,
  mockArchiveWorldEntry,
} from "@/fixtures/mocks/archive.mocks";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { mockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";

const INSTALLATION: string = "C:\\game";

/** What the open form sends for a game folder. */
function mockInstallationRoots(): XrayRoots {
  return { asset: null, roots: [{ mode: "auto", path: INSTALLATION }] };
}

describe("ArchivesService world subject", () => {
  it("opens a game folder as the roots the form named", async () => {
    setMockInvokeResponses({ ["plugin:archives|open_world"]: mockSessionResponse(mockArchivesWorldSubject()) });

    const { service } = mockInjectedService(ArchivesService);

    await service.openWorld(mockInstallationRoots());

    expect(mockInvoke).toHaveBeenCalledWith("plugin:archives|open_world", {
      sessionId: expect.any(String),
      roots: mockInstallationRoots(),
    });
    expect(service.subject.value?.kind).toBe(EArchiveSubject.WORLD);
    expect(service.entries.map((entry) => entry.name)).toEqual(["configs\\system.ltx", "scripts\\actor.script"]);
  });

  it("reads and extracts through the same commands a volume set uses", async () => {
    // The subject is the backend's, so a surface never picks a command by which kind is open. That dispatch used to
    // live here, once per operation, and each copy was a chance to send a world gesture down the archive path.
    const entry: ArchiveWorldEntry = mockArchiveWorldEntry();
    const result: ArchiveReadResult = { name: entry.name, content: "[system]", size: 8 };

    setMockInvokeResponses({
      ["plugin:archives|open_world"]: mockSessionResponse(mockArchivesWorldSubject([entry])),
      ["plugin:archives|read_file"]: result,
      ["plugin:archives|extract_file"]: { name: entry.name, destination: "C:\\out\\system.ltx", size: 8 },
    });

    const { service } = mockInjectedService(ArchivesService);

    await service.openWorld(mockInstallationRoots());
    await service.selectArchiveFile(entry);
    await service.extractFile(entry, "C:\\out\\system.ltx");

    expect(service.content.value?.kind === "text" ? service.content.value.result : null).toEqual(result);
    expect(mockInvoke).toHaveBeenCalledWith("plugin:archives|extract_file", {
      sessionId: expect.any(String),
      name: entry.name,
      destination: "C:\\out\\system.ltx",
    });
  });

  it("asks what a world cannot reach, and not for a name table it has none of", async () => {
    setMockInvokeResponses({
      ["plugin:archives|open_world"]: mockSessionResponse(mockArchivesWorldSubject()),
      ["plugin:archives|list_collisions"]: [],
    });

    const { service } = mockInjectedService(ArchivesService);

    await service.openWorld(mockInstallationRoots());

    expect(mockInvoke).toHaveBeenCalledWith("plugin:archives|list_collisions", { sessionId: expect.any(String) });
    expect(mockInvoke).not.toHaveBeenCalledWith("plugin:archives|list_shared_payloads", expect.anything());
  });

  it("publishes where the selected file came from and what it hides", async () => {
    const entry: ArchiveWorldEntry = mockArchiveWorldEntry({ shadowed: [mockArchivedContainer()] });

    setMockInvokeResponses({ ["plugin:archives|open_world"]: mockSessionResponse(mockArchivesWorldSubject([entry])) });

    const { service } = mockInjectedService(ArchivesService);

    await service.openWorld(mockInstallationRoots());
    await service.selectArchiveFile(entry);

    expect(service.selectedWorldEntry).toStrictEqual(entry);
    // A world has no name table, so the panel that reads one must find nothing rather than a fabricated descriptor.
    expect(service.selectedDescriptor).toBeNull();
  });

  it("replaces a volume set without closing anything, because one subject is open at a time", async () => {
    const closed: Array<string> = [];

    setMockInvokeResponses({
      ["plugin:archives|open_volumes"]: mockSessionResponse(mockArchivesVolumes([])),
      ["plugin:archives|open_world"]: mockSessionResponse(mockArchivesWorldSubject()),
      ["plugin:archives|list_collisions"]: [],
      ["plugin:archives|list_shared_payloads"]: [],
      ["plugin:archives|close_subject"]: () => void closed.push("subject"),
    });

    const { service } = mockInjectedService(ArchivesService);

    await service.openVolumes("C:\\game\\db");
    await service.openWorld(mockInstallationRoots());

    expect(service.subject.value?.kind).toBe(EArchiveSubject.WORLD);
    expect(closed).toEqual([]);
  });

  it("restores whichever subject the backend still holds", async () => {
    setMockInvokeResponses({
      ["plugin:archives|get_subject"]: mockSessionResponse(mockArchivesWorldSubject()),
      ["plugin:archives|list_collisions"]: [],
    });

    const { service } = mockInjectedService(ArchivesService);

    await service.onProvision(1);

    expect(service.subject.value?.kind).toBe(EArchiveSubject.WORLD);
  });
});
