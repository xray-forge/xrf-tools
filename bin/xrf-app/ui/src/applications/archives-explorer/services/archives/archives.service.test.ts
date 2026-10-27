import { describe, expect, it } from "@jest/globals";
import { isComputedProp } from "@wirestate/mobx";
import { EArchivesCommand } from "@xrf/ipc/commands/archives";
import { ESessionsCommand } from "@xrf/ipc/commands/sessions";
import { EVisualsCommand } from "@xrf/ipc/commands/visuals";
import { SessionScopeService } from "@xrf/ipc/session";
import { mockInvoke, mockIpcCommands } from "@xrf/ipc/testing";
import { ArchiveFileDescriptor } from "@xrf/ipc/types/xrf-archive";
import { XrayPathCollision } from "@xrf/ipc/types/xrf-vfs";

import { ArchivesService } from "@/applications/archives-explorer/services/archives/index";
import {
  mockArchiveFileDescriptor,
  mockArchivesVolumes,
  mockPathCollision,
} from "@/core/archive/fixtures/archive.mocks";
import { mockHeldSessionRoot } from "@/core/session/fixtures/session-root.mocks";
import { mockSessionRead, mockSessionResponse, mockSessionSnapshot } from "@/fixtures/mocks/session.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";

function mockArchivesService(files: Array<ArchiveFileDescriptor>): ArchivesService {
  const { service } = mockInjectedService(ArchivesService);

  mockHeldSessionRoot(service.subject, mockSessionSnapshot(mockArchivesVolumes(files)));

  return service;
}

describe("ArchivesService subject", () => {
  it("registers derived state as MobX computed properties", () => {
    const service: ArchivesService = mockArchivesService([]);

    expect(isComputedProp(service, "entries")).toBe(true);
    expect(isComputedProp(service, "isWriting")).toBe(true);
  });

  it("publishes the files of a project without the directories its volumes record", () => {
    // Every surface reads this one, so the rule is applied once per opened project rather than once per render.
    const service: ArchivesService = mockArchivesService([
      mockArchiveFileDescriptor({ name: "meshes", sizeCompressed: 0, sizeReal: 0 }),
      mockArchiveFileDescriptor({ name: "meshes\\actors\\stalker.ogf" }),
    ]);

    expect(service.entries.map((descriptor) => descriptor.name)).toEqual(["meshes\\actors\\stalker.ogf"]);
  });

  it("has no files before a project is opened", () => {
    const { service } = mockInjectedService(ArchivesService);

    expect(service.entries).toEqual([]);
  });

  it("stays restored with nothing held once the project closes", async () => {
    mockIpcCommands({ [EArchivesCommand.CLOSE_SUBJECT]: undefined });

    const service: ArchivesService = mockArchivesService([mockArchiveFileDescriptor({ name: "textures\\ui.dds" })]);

    await service.closeSubject();

    // Closed is still restored, with nothing held: the explorer shows its open form, not its loader.
    expect(service.subject.isRestored).toBe(true);
    expect(service.subject.value).toBeNull();
    expect(service.entries).toEqual([]);
  });
});

describe("ArchivesService visual preview lifecycle", () => {
  it("closes only its archive; the preview owns its model", async () => {
    // The preview goes through the shared visuals session, so a model left selected there is the one the visuals
    // explorer restores next time it is opened - which is the leak issue 0010 records.
    const invoked: Array<string> = [];

    mockIpcCommands({
      [EArchivesCommand.CLOSE_SUBJECT]: () => void invoked.push(EArchivesCommand.CLOSE_SUBJECT),
      [EVisualsCommand.CLOSE_MODEL]: () => void invoked.push(EVisualsCommand.CLOSE_MODEL),
    });

    await mockArchivesService([]).closeSubject();

    expect(invoked).toEqual([EArchivesCommand.CLOSE_SUBJECT]);
  });

  it("leaves its document to the tool's scope, released once, as the application deactivates", async () => {
    const invoked: Array<string> = [];

    mockIpcCommands({
      [EArchivesCommand.CLOSE_SUBJECT]: () => void invoked.push(EArchivesCommand.CLOSE_SUBJECT),
      [EVisualsCommand.CLOSE_MODEL]: () => void invoked.push(EVisualsCommand.CLOSE_MODEL),
      [ESessionsCommand.RELEASE_SCOPE]: () => {
        invoked.push(ESessionsCommand.RELEASE_SCOPE);

        return 1;
      },
    });

    const { service, container } = mockInjectedService(ArchivesService);

    mockHeldSessionRoot(service.subject, mockSessionSnapshot(mockArchivesVolumes([])));
    container.get(SessionScopeService).release();
    // Retired with the scope, the document is no longer the service's to close.
    await service.closeSubject();

    expect(invoked).toEqual([ESessionsCommand.RELEASE_SCOPE]);
  });
});

describe("ArchivesService reachability", () => {
  it("asks what the opened volume set overrides and what it cannot reach, in one fold", async () => {
    const collision: XrayPathCollision = mockPathCollision();

    mockIpcCommands({
      [EArchivesCommand.OPEN_VOLUMES]: mockSessionResponse(mockArchivesVolumes([])),
      [EArchivesCommand.LIST_OVERRIDES]: mockSessionRead({ overridden: [], unreachable: [collision] }),
    });

    const { service } = mockInjectedService(ArchivesService);

    await service.openVolumes("C:\\game\\database");

    expect(service.unreachable).toEqual([collision]);

    // One command for both halves: a volume set walks its whole merged name table to answer either, so asking
    // separately folded an installation-sized table twice.
    expect(mockInvoke).toHaveBeenCalledWith(EArchivesCommand.LIST_OVERRIDES, { sessionId: expect.any(String) });
  });

  it("keeps a project browsable when reachability cannot be answered", async () => {
    mockIpcCommands({
      [EArchivesCommand.OPEN_VOLUMES]: mockSessionResponse(mockArchivesVolumes([])),
      [EArchivesCommand.LIST_OVERRIDES]: mockSessionRead(() => {
        throw new Error("fold failed");
      }),
    });

    const { service } = mockInjectedService(ArchivesService);

    await service.openVolumes("C:\\game\\database");

    expect(service.subject.value).not.toBeNull();
    expect(service.subject.error).toBeNull();
    expect(service.overrides.error?.message).toBe("fold failed");
  });

  it("forgets them when the project closes", async () => {
    mockIpcCommands({
      [EArchivesCommand.OPEN_VOLUMES]: mockSessionResponse(mockArchivesVolumes([])),
      [EArchivesCommand.LIST_OVERRIDES]: mockSessionRead({ overridden: [], unreachable: [mockPathCollision()] }),
      [EArchivesCommand.CLOSE_SUBJECT]: undefined,
    });

    const { service } = mockInjectedService(ArchivesService);

    await service.openVolumes("C:\\game\\database");
    await service.closeSubject();

    expect(service.unreachable).toEqual([]);
  });
});
