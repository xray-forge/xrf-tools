import { describe, expect, it } from "@jest/globals";
import { flowResult, isComputedProp } from "@wirestate/mobx";
import { EArchivesCommand } from "@xrf/ipc/commands/archives";
import { mockInvoke, mockIpcCommands } from "@xrf/ipc/testing";
import { ArchiveFileDescription } from "@xrf/ipc/types/xrf-app-archives";
import { ArchiveFileDescriptor, ArchiveReadResult } from "@xrf/ipc/types/xrf-archive";
import { EPathEntryKind } from "@xrf/path";

import {
  IArchivesExplorerMock,
  mockArchivesExplorer,
} from "@/applications/archives-explorer/fixtures/archives-explorer.mocks";
import { mockArchiveFileDescriptor, mockArchivesVolumes } from "@/core/archive/fixtures/archive.mocks";
import { mockHeldSessionRoot } from "@/core/session/fixtures/session-root.mocks";
import { mockSessionRead, mockSessionSnapshot } from "@/fixtures/mocks/session.mocks";
import { deferred } from "@/fixtures/utils/promise";

import { ArchivePreviewService } from "./archive-preview.service";

function mockExplorer(files: Array<ArchiveFileDescriptor>): IArchivesExplorerMock {
  return mockArchivesExplorer(mockSessionSnapshot(mockArchivesVolumes(files)));
}

function mockService(files: Array<ArchiveFileDescriptor>): ArchivePreviewService {
  return mockExplorer(files).previewService;
}

describe("ArchivePreviewService", () => {
  it("registers derived state as MobX computed properties", () => {
    const service: ArchivePreviewService = mockService([]);

    expect(isComputedProp(service, "selection")).toBe(true);
    expect(isComputedProp(service, "content")).toBe(true);
    expect(isComputedProp(service, "selectedEntry")).toBe(true);
  });

  it("loads supported selected files", async () => {
    const descriptor = mockArchiveFileDescriptor();
    const result: ArchiveReadResult = { name: descriptor.name, content: "[system]", size: 8 };

    mockIpcCommands({ [EArchivesCommand.READ_FILE]: mockSessionRead(result) });

    const service: ArchivePreviewService = mockService([descriptor]);

    await service.selectFile(descriptor);

    expect(service.selectedEntry).toStrictEqual(descriptor);
    expect(service.content.value?.kind === "text" ? service.content.value.result : null).toEqual(result);
    expect(mockInvoke).toHaveBeenCalledWith(EArchivesCommand.READ_FILE, {
      sessionId: expect.any(String),
      name: descriptor.name,
    });
  });

  it("selects unsupported files without invoking the read command", async () => {
    const descriptor = mockArchiveFileDescriptor({ name: "meshes\\actor.ogf" });
    const service: ArchivePreviewService = mockService([descriptor]);

    await service.selectFile(descriptor);

    expect(service.selectedEntry).toStrictEqual(descriptor);
    expect(service.content.value).toBeNull();
    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it("asks the backend to describe an extension it cannot read as text", async () => {
    const description: ArchiveFileDescription = {
      scope: { kind: "volumes", volumes: 1 },
      format: { kind: "unsupported", reason: { kind: "noDescriber", extension: "omf" } },
    };

    mockIpcCommands({ [EArchivesCommand.DESCRIBE_FILE]: mockSessionRead(description) });

    const descriptor = mockArchiveFileDescriptor({ name: "meshes\\actor.omf" });
    const service: ArchivePreviewService = mockService([descriptor]);

    await service.selectFile(descriptor);

    expect(service.content.value).toEqual({ kind: "description", description });
    expect(mockInvoke).toHaveBeenCalledWith(EArchivesCommand.DESCRIBE_FILE, {
      sessionId: expect.any(String),
      name: descriptor.name,
    });
  });

  it("selects the entry a description referenced", () => {
    const referenced = mockArchiveFileDescriptor({ name: "textures\\act\\act_arm_1.dds" });
    const service: ArchivePreviewService = mockService([mockArchiveFileDescriptor(), referenced]);

    service.selectFileByName(referenced.name);

    expect(service.selectedEntry).toStrictEqual(referenced);
  });

  it("leaves the selection alone when a referenced entry is no longer listed", () => {
    const service: ArchivePreviewService = mockService([mockArchiveFileDescriptor()]);

    service.selectFileByName("textures\\act\\act_arm_1.dds");

    expect(service.selectedEntry).toBeNull();
  });

  it("allows only the latest selection to publish a completed read", async () => {
    const first = mockArchiveFileDescriptor({ name: "configs\\first.ltx" });
    const second = mockArchiveFileDescriptor({ name: "configs\\second.ltx" });
    const firstResult = deferred<ArchiveReadResult>();
    const secondResult = deferred<ArchiveReadResult>();

    mockIpcCommands({
      [EArchivesCommand.READ_FILE]: mockSessionRead((args?: Record<string, unknown>) =>
        args?.name === first.name ? firstResult.promise : secondResult.promise
      ),
    });

    const service: ArchivePreviewService = mockService([first, second]);
    const firstRead: Promise<void> = flowResult(service.selectFile(first));
    const secondRead: Promise<void> = flowResult(service.selectFile(second));

    secondResult.resolve({ name: second.name, content: "second", size: 6 });
    await secondRead;
    firstResult.resolve({ name: first.name, content: "first", size: 5 });
    await firstRead;

    expect(service.selectedEntry).toStrictEqual(second);
    expect(service.content.value?.kind === "text" ? service.content.value.result.content : null).toBe("second");
  });

  it("reads nothing before a subject is open", async () => {
    const { previewService } = mockArchivesExplorer();

    await previewService.selectFile(mockArchiveFileDescriptor());

    expect(previewService.content.value).toBeNull();
    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it("reads nothing as a write and leaves writing to the explorer's own check", () => {
    // A read is superseded by the next open, so treating it as busy is what dropped the second gesture entirely.
    const descriptor = mockArchiveFileDescriptor({ name: "configs\\system.ltx" });

    mockIpcCommands({
      // Neither settles, so both stay in flight for the length of the assertion.
      [EArchivesCommand.READ_FILE]: mockSessionRead(() => new Promise(() => {})),
      [EArchivesCommand.EXTRACT_FILE]: () => new Promise(() => {}),
    });

    const { archivesService, previewService } = mockExplorer([descriptor]);

    void previewService.selectFile(descriptor);

    expect(previewService.content.isLoading).toBe(true);
    expect(archivesService.isWriting).toBe(false);

    void archivesService.extractFile(descriptor, "C:\\out\\system.ltx");

    expect(archivesService.isWriting).toBe(true);
  });

  it("shows nothing of a subject closed since the selection was made", async () => {
    const descriptor = mockArchiveFileDescriptor({ name: "textures\\ui.dds" });

    mockIpcCommands({ [EArchivesCommand.CLOSE_SUBJECT]: undefined });

    const { archivesService, previewService } = mockExplorer([descriptor]);

    await previewService.selectFile(descriptor);
    await archivesService.closeSubject();

    expect(previewService.selection).toEqual({ kind: "none" });
    expect(previewService.content.value).toBeNull();
  });

  it("shows nothing of a subject another opened over", async () => {
    const descriptor = mockArchiveFileDescriptor({ name: "configs\\system.ltx" });
    const { archivesService, previewService } = mockExplorer([descriptor]);

    previewService.selectDirectory("configs");
    mockHeldSessionRoot(archivesService.subject, mockSessionSnapshot(mockArchivesVolumes([descriptor]), "another"));

    expect(previewService.selection).toEqual({ kind: "none" });
  });

  it("keeps the selection when closing fails, as the subject stays open", async () => {
    const descriptor = mockArchiveFileDescriptor({ name: "textures\\ui.dds" });

    mockIpcCommands({
      [EArchivesCommand.CLOSE_SUBJECT]: () => {
        throw new Error("archive is busy");
      },
    });

    const { archivesService, previewService } = mockExplorer([descriptor]);

    await previewService.selectFile(descriptor);
    await archivesService.closeSubject();

    expect(archivesService.subject.value).not.toBeNull();
    expect(previewService.selectedEntry).toStrictEqual(descriptor);
  });

  it("keeps file and directory selection mutually exclusive, the tree root an empty path", async () => {
    const entry = mockArchiveFileDescriptor({ name: "configs\\system.ltx" });
    const service: ArchivePreviewService = mockService([entry]);

    service.selectDirectory("");
    expect(service.selection).toEqual({ kind: EPathEntryKind.DIRECTORY, path: "" });

    // Both being set at once would leave the content area with two things claiming to be selected.
    await service.selectFile(entry);
    expect(service.selection).toEqual({ kind: EPathEntryKind.FILE, entry });
    expect(service.selectedEntry).toEqual(entry);

    service.selectDirectory("configs");
    expect(service.selection).toEqual({ kind: EPathEntryKind.DIRECTORY, path: "configs" });
    expect(service.selectedEntry).toBeNull();
  });

  it("forgets how the last writes ended as a directory is selected", () => {
    const { archivesService, previewService } = mockExplorer([]);

    previewService.selectDirectory("configs");

    expect(previewService.selection).toEqual({ kind: EPathEntryKind.DIRECTORY, path: "configs" });
    expect(archivesService.extraction.result).toBeNull();
    expect(archivesService.fileExtraction.value).toBeNull();
  });
});
