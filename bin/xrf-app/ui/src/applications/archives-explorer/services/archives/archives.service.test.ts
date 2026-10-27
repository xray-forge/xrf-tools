import { describe, expect, it } from "@jest/globals";
import { flowResult, isComputedProp } from "@wirestate/mobx";
import { EArchivesCommand } from "@xrf/ipc/commands/archives";
import { ESessionsCommand } from "@xrf/ipc/commands/sessions";
import { EVisualsCommand } from "@xrf/ipc/commands/visuals";
import { SessionScopeService } from "@xrf/ipc/session";
import { mockInvoke, mockIpcCommands } from "@xrf/ipc/testing";
import { ArchiveFileDescription } from "@xrf/ipc/types/xrf-app-archives";
import { ArchiveFileDescriptor, ArchiveReadResult } from "@xrf/ipc/types/xrf-archive";
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

function ignoreReadResult(): void {}

function mockArchivesService(files: Array<ArchiveFileDescriptor>): ArchivesService {
  const { service } = mockInjectedService(ArchivesService);

  mockHeldSessionRoot(service.subject, mockSessionSnapshot(mockArchivesVolumes(files)));

  return service;
}

describe("ArchivesService file selection", () => {
  it("registers derived state as MobX computed properties", () => {
    const service: ArchivesService = mockArchivesService([]);

    expect(isComputedProp(service, "entries")).toBe(true);
    expect(isComputedProp(service, "selectedEntry")).toBe(true);
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

  it("loads supported selected files", async () => {
    const descriptor = mockArchiveFileDescriptor();
    const result: ArchiveReadResult = { name: descriptor.name, content: "[system]", size: 8 };

    mockIpcCommands({ [EArchivesCommand.READ_FILE]: mockSessionRead(result) });

    const service: ArchivesService = mockArchivesService([descriptor]);

    await service.selectArchiveFile(descriptor);

    expect(service.selectedEntry).toStrictEqual(descriptor);
    expect(service.content.value?.kind === "text" ? service.content.value.result : null).toEqual(result);
    expect(mockInvoke).toHaveBeenCalledWith(EArchivesCommand.READ_FILE, {
      sessionId: expect.any(String),
      name: descriptor.name,
    });
  });

  it("selects unsupported files without invoking the read command", async () => {
    const descriptor = mockArchiveFileDescriptor({ name: "meshes\\actor.ogf" });
    const service: ArchivesService = mockArchivesService([descriptor]);

    await service.selectArchiveFile(descriptor);

    expect(service.selectedEntry).toStrictEqual(descriptor);
    expect(service.content.value?.kind === "text" ? service.content.value.result : null).toBeNull();
    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it("asks the backend to describe an extension it cannot read as text", async () => {
    const description: ArchiveFileDescription = {
      scope: { kind: "volumes", volumes: 1 },
      format: { kind: "unsupported", reason: { kind: "noDescriber", extension: "omf" } },
    };

    mockIpcCommands({ [EArchivesCommand.DESCRIBE_FILE]: mockSessionRead(description) });

    const descriptor = mockArchiveFileDescriptor({ name: "meshes\\actor.omf" });
    const service: ArchivesService = mockArchivesService([descriptor]);

    await service.selectArchiveFile(descriptor);

    expect(service.content.value).toEqual({ kind: "description", description });
    expect(mockInvoke).toHaveBeenCalledWith(EArchivesCommand.DESCRIBE_FILE, {
      sessionId: expect.any(String),
      name: descriptor.name,
    });
  });

  it("opens the entry a description referenced", () => {
    const referenced = mockArchiveFileDescriptor({ name: "textures\\act\\act_arm_1.dds" });
    const service: ArchivesService = mockArchivesService([mockArchiveFileDescriptor(), referenced]);

    service.openArchiveFileByName(referenced.name);

    expect(service.selectedEntry).toStrictEqual(referenced);
  });

  it("leaves the selection alone when a referenced entry is no longer listed", () => {
    const descriptor = mockArchiveFileDescriptor();
    const service: ArchivesService = mockArchivesService([descriptor]);

    service.openArchiveFileByName("textures\\act\\act_arm_1.dds");

    expect(service.selectedEntry).toBeNull();
  });

  it("allows only the latest selection to publish a completed read", async () => {
    const first = mockArchiveFileDescriptor({ name: "configs\\first.ltx" });
    const second = mockArchiveFileDescriptor({ name: "configs\\second.ltx" });
    let resolveFirst: (value: ArchiveReadResult) => void = ignoreReadResult;
    let resolveSecond: (value: ArchiveReadResult) => void = ignoreReadResult;
    const firstResult: Promise<ArchiveReadResult> = new Promise((resolve) => {
      resolveFirst = resolve;
    });
    const secondResult: Promise<ArchiveReadResult> = new Promise((resolve) => {
      resolveSecond = resolve;
    });

    mockIpcCommands({
      [EArchivesCommand.READ_FILE]: mockSessionRead((args?: Record<string, unknown>) =>
        args?.name === first.name ? firstResult : secondResult
      ),
    });

    const service: ArchivesService = mockArchivesService([first, second]);
    const firstRead: Promise<void> = flowResult(service.selectArchiveFile(first));
    const secondRead: Promise<void> = flowResult(service.selectArchiveFile(second));

    resolveSecond({ name: second.name, content: "second", size: 6 });
    await secondRead;
    resolveFirst({ name: first.name, content: "first", size: 5 });
    await firstRead;

    expect(service.selectedEntry).toStrictEqual(second);
    expect(service.content.value?.kind === "text" ? service.content.value.result.name : null).toBe(second.name);
    expect(service.content.value?.kind === "text" ? service.content.value.result.content : null).toBe("second");
  });

  it("reports a write in flight as busy and a read as free", () => {
    // The one thing a surface holding an open back is allowed to test: a read is superseded by the next
    // open, so treating it as busy is what dropped the second gesture entirely.
    const descriptor = mockArchiveFileDescriptor({ name: "configs\\system.ltx" });

    mockIpcCommands({
      // Neither settles, so both stay in flight for the length of the assertion.
      [EArchivesCommand.READ_FILE]: mockSessionRead(() => new Promise(() => {})),
      [EArchivesCommand.EXTRACT_FILE]: () => new Promise(() => {}),
    });

    const service: ArchivesService = mockArchivesService([descriptor]);

    void service.selectArchiveFile(descriptor);

    expect(service.content.isLoading).toBe(true);
    expect(service.isWriting).toBe(false);

    void service.extractFile(descriptor, "C:\\out\\system.ltx");

    expect(service.isWriting).toBe(true);
  });

  it("clears file state when the project closes", async () => {
    const descriptor = mockArchiveFileDescriptor({ name: "textures\\ui.dds" });

    mockIpcCommands({ [EArchivesCommand.CLOSE_SUBJECT]: undefined });

    const service: ArchivesService = mockArchivesService([descriptor]);

    await service.selectArchiveFile(descriptor);
    await service.closeSubject();

    // Closed is still restored, with nothing held: the explorer shows its open form, not its loader.
    expect(service.subject.isRestored).toBe(true);
    expect(service.subject.value).toBeNull();
    expect(service.selectedEntry).toBeNull();
    expect(service.content.value?.kind === "text" ? service.content.value.result : null).toBeNull();
  });

  it("preserves the open project and selection when closing fails", async () => {
    const descriptor = mockArchiveFileDescriptor({ name: "textures\\ui.dds" });

    mockIpcCommands({
      [EArchivesCommand.CLOSE_SUBJECT]: () => {
        throw new Error("archive is busy");
      },
    });

    const service: ArchivesService = mockArchivesService([descriptor]);

    await service.selectArchiveFile(descriptor);

    // A close that fails is told as a notice; the subject is still open, so what is shown of it stays.
    await service.closeSubject();

    expect(service.subject.value).not.toBeNull();
    expect(service.selectedEntry).toStrictEqual(descriptor);
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
