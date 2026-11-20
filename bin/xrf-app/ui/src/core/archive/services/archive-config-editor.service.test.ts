import { beforeEach, describe, expect, it } from "@jest/globals";
import { EventBus, Injectable } from "@wirestate/core";
import { flowResult } from "@wirestate/mobx";
import { BYTES_PER_MEGABYTE } from "@xrf/format";
import { resetMockInvoke } from "@xrf/ipc/testing";
import { ARCHIVES_JOB_PACK, ARCHIVES_JOB_UNPACK } from "@xrf/ipc/types/xrf-app-archives";
import { Logger } from "@xrf/logging";

import {
  ArchiveConfigEditorService,
  IArchiveEditedConfig,
} from "@/core/archive/services/archive-config-editor.service";
import { IJobSettledPayload, JOB_SETTLED_EVENT } from "@/core/jobs/lib";
import { JobsService } from "@/core/jobs/services/jobs.service";
import { IInjectedServiceMockDescriptor, mockInjectedService } from "@/fixtures/utils/container";
import { deferred } from "@/fixtures/utils/promise";

enum ETestSection {
  FIRST = "first",
  SECOND = "second",
}

interface ITestConfig extends IArchiveEditedConfig {
  rule: string;
  // Never written to a file, as the run's paths are not.
  path: string;
}

const CONFIG: ITestConfig = {
  name: "test",
  header: null,
  mode: "Compress",
  maxVolumeSize: 1900 * BYTES_PER_MEGABYTE,
  volumeExtension: "Db",
  rule: "configs",
  path: "",
};

@Injectable()
class TestConfigEditorService extends ArchiveConfigEditorService<ITestConfig, string, ETestSection> {
  public readonly files: Map<string, Partial<ITestConfig>> = new Map();
  public readonly exported: Array<string> = [];
  public listing: (config: ITestConfig) => Promise<Array<string>> = () => Promise.resolve([]);

  public constructor() {
    super(new Logger(__MODULE_NAME__), [ARCHIVES_JOB_PACK], ETestSection.FIRST);
  }

  protected override onExported(path: string): void {
    this.exported.push(path);
  }

  protected override toSavedState(config: ITestConfig): string {
    return JSON.stringify({ ...config, path: null });
  }

  protected override readConfig(path: string, current: ITestConfig): Promise<ITestConfig> {
    const file: Partial<ITestConfig> | undefined = this.files.get(path);

    return file ? Promise.resolve({ ...current, ...file }) : Promise.reject(new Error(`No configuration at ${path}`));
  }

  protected override writeConfig(path: string, config: ITestConfig): Promise<unknown> {
    if (path.endsWith(".locked")) {
      return Promise.reject(new Error(`Cannot write ${path}`));
    }

    this.files.set(path, config);

    return Promise.resolve(null);
  }

  protected override listVolumes(config: ITestConfig): Promise<Array<string>> {
    return this.listing(config);
  }
}

function mockEditor(): IInjectedServiceMockDescriptor<TestConfigEditorService> {
  const descriptor: IInjectedServiceMockDescriptor<TestConfigEditorService> =
    mockInjectedService(TestConfigEditorService);

  descriptor.service.config = { ...CONFIG };

  return descriptor;
}

function emitSettled(
  descriptor: IInjectedServiceMockDescriptor<TestConfigEditorService>,
  payload: IJobSettledPayload
): void {
  descriptor.container.get(EventBus).emit<IJobSettledPayload>(JOB_SETTLED_EVENT, payload);
}

describe("ArchiveConfigEditorService sections", () => {
  it("opens on the section its subclass names, and opens another when asked", () => {
    const { service } = mockEditor();

    expect(service.section).toBe(ETestSection.FIRST);

    service.openSection(ETestSection.SECOND);

    expect(service.section).toBe(ETestSection.SECOND);
  });
});

describe("ArchiveConfigEditorService configuration files", () => {
  beforeEach(resetMockInvoke);

  it("reads a file over the open configuration and remembers it", async () => {
    const { service } = mockEditor();

    service.files.set("C:\\work\\rules.ltx", { rule: "scripts" });

    await service.importConfig("C:\\work\\rules.ltx");

    expect(service.config).toEqual({ ...CONFIG, rule: "scripts" });
    expect(service.configPath).toBe("C:\\work\\rules.ltx");
    expect(service.configName).toBe("rules.ltx");
    expect(service.isBusy).toBe(false);
  });

  it("holds the busy lane while a file is read", async () => {
    const { service } = mockEditor();

    service.files.set("rules.ltx", { rule: "scripts" });

    const importing = service.importConfig("rules.ltx");

    expect(service.isBusy).toBe(true);

    await importing;

    expect(service.isBusy).toBe(false);
  });

  it("keeps the open configuration and its file when a read fails", async () => {
    const { service } = mockEditor();

    await service.importConfig("missing.ltx");

    expect(service.error).toBe("No configuration at missing.ltx");
    expect(service.config).toEqual(CONFIG);
    expect(service.configPath).toBeNull();
    expect(service.isBusy).toBe(false);
  });

  it("writes the open configuration, remembers the file and tells the subclass", async () => {
    const { service } = mockEditor();

    service.patchConfig({ rule: "scripts" });

    await service.exportConfig("C:\\work\\rules.ltx");

    expect(service.files.get("C:\\work\\rules.ltx")).toEqual({ ...CONFIG, rule: "scripts" });
    expect(service.configName).toBe("rules.ltx");
    expect(service.exported).toEqual(["C:\\work\\rules.ltx"]);
    expect(service.isDirty).toBe(false);
  });

  it("reports a failed write without remembering the file or telling the subclass", async () => {
    const { service } = mockEditor();

    await service.exportConfig("rules.locked");

    expect(service.error).toBe("Cannot write rules.locked");
    expect(service.configPath).toBeNull();
    expect(service.exported).toEqual([]);
  });

  it("clears a failed read once the next one succeeds", async () => {
    const { service } = mockEditor();

    service.files.set("rules.ltx", {});

    await service.importConfig("missing.ltx");
    await service.importConfig("rules.ltx");

    expect(service.error).toBeNull();
  });

  it("reads and writes nothing while a run of its kinds is in flight", async () => {
    const { service, container } = mockEditor();

    container.get(JobsService).jobs = [
      {
        id: "adopted-pack",
        kind: ARCHIVES_JOB_PACK,
        request: null,
        progress: null,
        isAdopted: true,
        isCancelRequested: false,
      },
    ];
    service.files.set("rules.ltx", { rule: "scripts" });

    await service.importConfig("rules.ltx");
    await service.exportConfig("written.ltx");

    expect(service.config).toEqual(CONFIG);
    expect(service.files.has("written.ltx")).toBe(false);
    expect(service.configPath).toBeNull();
  });
});

describe("ArchiveConfigEditorService edits", () => {
  beforeEach(resetMockInvoke);

  it("counts nothing as unsaved until a file has been read or written", () => {
    const { service } = mockEditor();

    service.patchConfig({ rule: "scripts" });

    expect(service.isDirty).toBe(false);
  });

  it("counts an edit after a read as unsaved, and editing it back as saved", async () => {
    const { service } = mockEditor();

    service.files.set("rules.ltx", {});

    await service.importConfig("rules.ltx");

    service.patchConfig({ rule: "scripts" });

    expect(service.isDirty).toBe(true);

    service.patchConfig({ rule: CONFIG.rule });

    expect(service.isDirty).toBe(false);
  });

  it("does not count what its subclass leaves out of a file as an edit", async () => {
    const { service } = mockEditor();

    service.files.set("rules.ltx", {});

    await service.importConfig("rules.ltx");

    service.patchConfig({ path: "C:\\out" });

    expect(service.isDirty).toBe(false);
  });

  it("ignores an edit while no configuration is open", () => {
    const { service } = mockInjectedService(TestConfigEditorService);

    service.patchConfig({ rule: "scripts" });

    expect(service.config).toBeNull();
  });

  it("clears a configuration failure when the configuration or the volume ceiling changes", async () => {
    const { service } = mockEditor();

    await service.importConfig("missing.ltx");

    service.patchConfig({ rule: "scripts" });

    expect(service.error).toBeNull();

    await service.importConfig("missing.ltx");

    service.setVolumeSize("512");

    expect(service.error).toBeNull();
    expect(service.volumeSizeBytes).toBe(512 * BYTES_PER_MEGABYTE);
  });
});

describe("ArchiveConfigEditorService runs", () => {
  beforeEach(resetMockInvoke);

  it("shows a configuration failure before a failed run's", async () => {
    const descriptor = mockEditor();
    const { service, container } = descriptor;

    container.provision();

    emitSettled(descriptor, {
      id: "failed-pack",
      kind: ARCHIVES_JOB_PACK,
      conclusion: "failed",
      error: "Could not finish",
      result: null,
    });

    expect(service.error).toBe("Could not finish");

    await service.importConfig("missing.ltx");

    expect(service.error).toBe("No configuration at missing.ltx");
  });

  it("clears a configuration failure when a run of its kinds settles unawaited", async () => {
    const descriptor = mockEditor();
    const { service, container } = descriptor;

    container.provision();

    await service.importConfig("missing.ltx");

    emitSettled(descriptor, {
      id: "adopted-pack",
      kind: ARCHIVES_JOB_PACK,
      conclusion: "completed",
      error: null,
      result: "packed",
    });

    expect(service.error).toBeNull();
    expect(service.operation.result).toBe("packed");
  });

  it("keeps a configuration failure when another kind of run settles", async () => {
    const descriptor = mockEditor();
    const { service, container } = descriptor;

    container.provision();

    await service.importConfig("missing.ltx");

    emitSettled(descriptor, {
      id: "adopted-unpack",
      kind: ARCHIVES_JOB_UNPACK,
      conclusion: "completed",
      error: null,
      result: null,
    });

    expect(service.error).toBe("No configuration at missing.ltx");
  });
});

describe("ArchiveConfigEditorService destination check", () => {
  beforeEach(resetMockInvoke);

  it("records the volumes the destination already holds", async () => {
    const { service } = mockEditor();

    service.listing = () => Promise.resolve(["C:\\out\\test.db0"]);

    await service.checkDestination({ ...CONFIG, path: "C:\\out" });

    expect(service.publishedVolumes).toEqual(["C:\\out\\test.db0"]);
  });

  it("reads a listing that fails as nothing found", async () => {
    const { service } = mockEditor();

    service.publishedVolumes = ["C:\\out\\test.db0"];
    service.listing = () => Promise.reject(new Error("Access denied"));

    await service.checkDestination(CONFIG);

    expect(service.publishedVolumes).toEqual([]);
  });

  it("keeps the newer listing when an older one answers after it", async () => {
    const older = deferred<Array<string>>();
    const { service } = mockEditor();

    service.listing = () => older.promise;

    const olderCheck = flowResult(service.checkDestination({ ...CONFIG, path: "C:\\old" }));

    service.listing = () => Promise.resolve(["C:\\new\\test.db0"]);

    await service.checkDestination({ ...CONFIG, path: "C:\\new" });

    older.resolve(["C:\\old\\test.db0"]);
    await olderCheck;

    expect(service.publishedVolumes).toEqual(["C:\\new\\test.db0"]);
  });
});
