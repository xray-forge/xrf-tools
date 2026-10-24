import { describe, expect, it } from "@jest/globals";
import { EArchivesCommand } from "@xrf/ipc/commands/archives";
import { mockInvoke, mockIpcCommands } from "@xrf/ipc/testing";
import { ArchiveWorldEntry, EArchiveSubject } from "@xrf/ipc/types/xrf-app-archives";
import { ArchiveReadResult } from "@xrf/ipc/types/xrf-archive";
import { XrayRoots } from "@xrf/ipc/types/xrf-vfs";

import { ArchivesService } from "@/applications/archives-explorer/services/archives/index";
import {
  mockArchiveShadowedCopy,
  mockArchivesVolumes,
  mockArchivesWorldSubject,
  mockArchiveWorldEntry,
} from "@/fixtures/mocks/archive.mocks";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";

const INSTALLATION: string = "C:\\game";

/** What the open form sends for a game folder. */
function mockInstallationRoots(): XrayRoots {
  return { asset: null, roots: [{ mode: "auto", path: INSTALLATION }] };
}

describe("ArchivesService world subject", () => {
  it("opens a game folder as the roots the form named", async () => {
    mockIpcCommands({ [EArchivesCommand.OPEN_WORLD]: mockSessionResponse(mockArchivesWorldSubject()) });

    const { service } = mockInjectedService(ArchivesService);

    await service.openWorld(mockInstallationRoots());

    expect(mockInvoke).toHaveBeenCalledWith(EArchivesCommand.OPEN_WORLD, {
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

    mockIpcCommands({
      [EArchivesCommand.OPEN_WORLD]: mockSessionResponse(mockArchivesWorldSubject([entry])),
      [EArchivesCommand.READ_FILE]: result,
      [EArchivesCommand.EXTRACT_FILE]: { name: entry.name, destination: "C:\\out\\system.ltx", size: 8 },
    });

    const { service } = mockInjectedService(ArchivesService);

    await service.openWorld(mockInstallationRoots());
    await service.selectArchiveFile(entry);
    await service.extractFile(entry, "C:\\out\\system.ltx");

    expect(service.content.value?.kind === "text" ? service.content.value.result : null).toEqual(result);
    expect(mockInvoke).toHaveBeenCalledWith(EArchivesCommand.EXTRACT_FILE, {
      sessionId: expect.any(String),
      name: entry.name,
      destination: "C:\\out\\system.ltx",
    });
  });

  it("asks what a world cannot reach, and not for a name table it has none of", async () => {
    mockIpcCommands({
      [EArchivesCommand.OPEN_WORLD]: mockSessionResponse(mockArchivesWorldSubject()),
      [EArchivesCommand.LIST_OVERRIDES]: { overridden: [], unreachable: [] },
    });

    const { service } = mockInjectedService(ArchivesService);

    await service.openWorld(mockInstallationRoots());

    expect(mockInvoke).toHaveBeenCalledWith(EArchivesCommand.LIST_OVERRIDES, { sessionId: expect.any(String) });
    expect(mockInvoke).not.toHaveBeenCalledWith(EArchivesCommand.LIST_SHARED_PAYLOADS, expect.anything());
  });

  it("publishes where the selected file came from and what it hides", async () => {
    const entry: ArchiveWorldEntry = mockArchiveWorldEntry({ shadowed: [mockArchiveShadowedCopy()] });

    mockIpcCommands({ [EArchivesCommand.OPEN_WORLD]: mockSessionResponse(mockArchivesWorldSubject([entry])) });

    const { service } = mockInjectedService(ArchivesService);

    await service.openWorld(mockInstallationRoots());
    await service.selectArchiveFile(entry);

    expect(service.selectedWorldEntry).toStrictEqual(entry);
    // A world has no name table, so the panel that reads one must find nothing rather than a fabricated descriptor.
    expect(service.selectedDescriptor).toBeNull();
  });

  it("replaces a volume set, closing only the volume set it replaced", async () => {
    const closed: Array<Array<string>> = [];

    mockIpcCommands({
      [EArchivesCommand.OPEN_VOLUMES]: mockSessionResponse(mockArchivesVolumes([])),
      [EArchivesCommand.OPEN_WORLD]: mockSessionResponse(mockArchivesWorldSubject()),
      [EArchivesCommand.LIST_OVERRIDES]: { overridden: [], unreachable: [] },
      [EArchivesCommand.LIST_SHARED_PAYLOADS]: [],
      [EArchivesCommand.CLOSE_SUBJECT]: (args?: Record<string, unknown>) =>
        void closed.push(args?.sessionIds as Array<string>),
    });

    const { service } = mockInjectedService(ArchivesService);

    await service.openVolumes("C:\\game\\db");

    const volumes: unknown = mockInvoke.mock.calls.find(([name]) => name === EArchivesCommand.OPEN_VOLUMES)?.[1]
      ?.sessionId;

    await service.openWorld(mockInstallationRoots());

    expect(service.subject.value?.kind).toBe(EArchiveSubject.WORLD);
    expect(closed).toEqual([[volumes]]);
  });

  it("restores whichever subject the backend still holds", async () => {
    mockIpcCommands({
      [EArchivesCommand.RESTORE_SUBJECT]: mockSessionResponse(mockArchivesWorldSubject()),
      [EArchivesCommand.LIST_OVERRIDES]: { overridden: [], unreachable: [] },
    });

    const { service } = mockInjectedService(ArchivesService);

    await service.onProvision(1);

    expect(service.subject.value?.kind).toBe(EArchiveSubject.WORLD);
  });
});
