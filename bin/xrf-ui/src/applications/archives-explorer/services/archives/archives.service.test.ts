import { describe, expect, it } from "@jest/globals";
import { flowResult, isComputedProp } from "@wirestate/mobx";

import { ArchivesService } from "@/applications/archives-explorer/services/archives/index";
import { ArchiveFileDescriptor, ProjectReadResult } from "@/core/bindings/types/xrf-archive";
import { XrayPathCollision } from "@/core/bindings/types/xrf-vfs";
import { mockArchiveFileDescriptor, mockArchivesProject, mockPathCollision } from "@/fixtures/mocks/archive.mocks";
import { mockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { createLoadable } from "@/lib/loadable";

function ignoreReadResult(): void {}

function mockArchivesService(files: Array<ArchiveFileDescriptor>): ArchivesService {
  const { service } = mockInjectedService(ArchivesService);

  service.project = createLoadable(mockArchivesProject(files));

  return service;
}

describe("ArchivesService file selection", () => {
  it("registers derived state as MobX computed properties", () => {
    const service: ArchivesService = mockArchivesService([]);

    expect(isComputedProp(service, "files")).toBe(true);
    expect(isComputedProp(service, "selectedFile")).toBe(true);
    expect(isComputedProp(service, "selectedDirectory")).toBe(true);
    expect(isComputedProp(service, "isWriting")).toBe(true);
  });

  it("publishes the files of a project without the directories its volumes record", () => {
    // Every surface reads this one, so the rule is applied once per opened project rather than once per render.
    const service: ArchivesService = mockArchivesService([
      mockArchiveFileDescriptor({ name: "meshes", sizeCompressed: 0, sizeReal: 0 }),
      mockArchiveFileDescriptor({ name: "meshes\\actors\\stalker.ogf" }),
    ]);

    expect(service.files.map((descriptor) => descriptor.name)).toEqual(["meshes\\actors\\stalker.ogf"]);
  });

  it("has no files before a project is opened", () => {
    const { service } = mockInjectedService(ArchivesService);

    expect(service.files).toEqual([]);
  });

  it("loads supported selected files", async () => {
    const descriptor = mockArchiveFileDescriptor();
    const result: ProjectReadResult = { name: descriptor.name, content: "[system]", size: 8 };

    setMockInvokeResponses({ ["plugin:archives|read_file"]: result });

    const service: ArchivesService = mockArchivesService([descriptor]);

    await service.selectArchiveFile(descriptor);

    expect(service.selectedFile).toStrictEqual(descriptor);
    expect(service.content.value?.kind === "text" ? service.content.value.result : null).toEqual(result);
    expect(mockInvoke).toHaveBeenCalledWith("plugin:archives|read_file", { path: descriptor.name });
  });

  it("selects unsupported files without invoking the read command", async () => {
    const descriptor = mockArchiveFileDescriptor({ extension: "ogf", name: "meshes\\actor.ogf" });
    const service: ArchivesService = mockArchivesService([descriptor]);

    await service.selectArchiveFile(descriptor);

    expect(service.selectedFile).toStrictEqual(descriptor);
    expect(service.content.value?.kind === "text" ? service.content.value.result : null).toBeNull();
    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it("allows only the latest selection to publish a completed read", async () => {
    const first = mockArchiveFileDescriptor({ name: "configs\\first.ltx" });
    const second = mockArchiveFileDescriptor({ name: "configs\\second.ltx" });
    let resolveFirst: (value: ProjectReadResult) => void = ignoreReadResult;
    let resolveSecond: (value: ProjectReadResult) => void = ignoreReadResult;
    const firstResult: Promise<ProjectReadResult> = new Promise((resolve) => {
      resolveFirst = resolve;
    });
    const secondResult: Promise<ProjectReadResult> = new Promise((resolve) => {
      resolveSecond = resolve;
    });

    setMockInvokeResponses({
      ["plugin:archives|read_file"]: (args?: Record<string, unknown>) =>
        args?.path === first.name ? firstResult : secondResult,
    });

    const service: ArchivesService = mockArchivesService([first, second]);
    const firstRead: Promise<void> = flowResult(service.selectArchiveFile(first));
    const secondRead: Promise<void> = flowResult(service.selectArchiveFile(second));

    resolveSecond({ name: second.name, content: "second", size: 6 });
    await secondRead;
    resolveFirst({ name: first.name, content: "first", size: 5 });
    await firstRead;

    expect(service.selectedFile).toStrictEqual(second);
    expect(service.content.value?.kind === "text" ? service.content.value.result.name : null).toBe(second.name);
    expect(service.content.value?.kind === "text" ? service.content.value.result.content : null).toBe("second");
  });

  it("reports a write in flight as busy and a read as free", () => {
    // The one thing a surface holding an open back is allowed to test: a read is superseded by the next
    // open, so treating it as busy is what dropped the second gesture entirely.
    const descriptor = mockArchiveFileDescriptor({ name: "configs\\system.ltx" });

    setMockInvokeResponses({
      // Neither settles, so both stay in flight for the length of the assertion.
      ["plugin:archives|read_file"]: () => new Promise(() => {}),
      ["plugin:archives|extract_file"]: () => new Promise(() => {}),
    });

    const service: ArchivesService = mockArchivesService([descriptor]);

    void service.selectArchiveFile(descriptor);

    expect(service.content.isLoading).toBe(true);
    expect(service.isWriting).toBe(false);

    void service.extractFile(descriptor, "C:\\out\\system.ltx");

    expect(service.isWriting).toBe(true);
  });

  it("clears file state when the project closes", async () => {
    const descriptor = mockArchiveFileDescriptor({ extension: "dds", name: "textures\\ui.dds" });

    setMockInvokeResponses({ ["plugin:archives|close_project"]: undefined });

    const service: ArchivesService = mockArchivesService([descriptor]);

    await service.selectArchiveFile(descriptor);
    await service.closeProject();

    expect(service.selectedFile).toBeNull();
    expect(service.content.value?.kind === "text" ? service.content.value.result : null).toBeNull();
  });

  it("clears file state when the project is reset", async () => {
    const descriptor = mockArchiveFileDescriptor({ extension: "dds", name: "textures\\ui.dds" });
    const service: ArchivesService = mockArchivesService([descriptor]);

    await service.selectArchiveFile(descriptor);
    service.resetArchivesProject();

    expect(service.selectedFile).toBeNull();
    expect(service.content.value?.kind === "text" ? service.content.value.result : null).toBeNull();
  });

  it("preserves the open project and selection when closing fails", async () => {
    const descriptor = mockArchiveFileDescriptor({ extension: "dds", name: "textures\\ui.dds" });

    setMockInvokeResponses({
      ["plugin:archives|close_project"]: () => {
        throw new Error("archive is busy");
      },
    });

    const service: ArchivesService = mockArchivesService([descriptor]);

    await service.selectArchiveFile(descriptor);

    await expect(service.closeProject()).rejects.toThrow("archive is busy");
    expect(service.selectedFile).toStrictEqual(descriptor);
  });
});

describe("ArchivesService visual preview lifecycle", () => {
  it("drops the model its preview parked when the project closes", async () => {
    // The preview goes through the shared visuals session, so a model left selected there is the one the visuals
    // explorer restores next time it is opened - which is the leak issue 0010 records.
    const invoked: Array<string> = [];

    setMockInvokeResponses({
      ["plugin:archives|close_project"]: () => void invoked.push("plugin:archives|close_project"),
      ["plugin:visuals|close_model"]: () => void invoked.push("plugin:visuals|close_model"),
    });

    await mockArchivesService([]).closeProject();

    expect(invoked).toEqual(["plugin:archives|close_project", "plugin:visuals|close_model"]);
  });

  it("drops it when the application deactivates too", () => {
    const invoked: Array<string> = [];

    setMockInvokeResponses({
      ["plugin:archives|close_project"]: () => void invoked.push("plugin:archives|close_project"),
      ["plugin:visuals|close_model"]: () => void invoked.push("plugin:visuals|close_model"),
    });

    mockArchivesService([]).onDeactivation();

    expect(invoked).toEqual(["plugin:archives|close_project", "plugin:visuals|close_model"]);
  });
});

describe("ArchivesService reachability", () => {
  it("asks what the opened volume set cannot reach", async () => {
    const collision: XrayPathCollision = mockPathCollision();

    setMockInvokeResponses({
      ["plugin:archives|open_project"]: mockArchivesProject([]),
      ["plugin:archives|list_collisions"]: [collision],
    });

    const { service } = mockInjectedService(ArchivesService);

    await service.openProject("C:\\game\\database");

    expect(service.collisions.value).toEqual([collision]);
    expect(mockInvoke).toHaveBeenCalledWith("plugin:archives|list_collisions");
  });

  it("keeps a project browsable when reachability cannot be answered", async () => {
    setMockInvokeResponses({
      ["plugin:archives|open_project"]: mockArchivesProject([]),
      ["plugin:archives|list_collisions"]: () => {
        throw new Error("fold failed");
      },
    });

    const { service } = mockInjectedService(ArchivesService);

    await service.openProject("C:\\game\\database");

    expect(service.project.value).not.toBeNull();
    expect(service.project.error).toBeNull();
    expect(service.collisions.error?.message).toBe("fold failed");
  });

  it("forgets them when the project closes", async () => {
    setMockInvokeResponses({
      ["plugin:archives|open_project"]: mockArchivesProject([]),
      ["plugin:archives|list_collisions"]: [mockPathCollision()],
      ["plugin:archives|close_project"]: undefined,
    });

    const { service } = mockInjectedService(ArchivesService);

    await service.openProject("C:\\game\\database");
    await service.closeProject();

    expect(service.collisions.value).toEqual([]);
  });
});
