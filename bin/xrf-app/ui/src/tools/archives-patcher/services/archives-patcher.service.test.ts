import { beforeEach, describe, expect, it } from "@jest/globals";
import { flowResult } from "@wirestate/mobx";
import { BYTES_PER_MEGABYTE } from "@xrf/format";
import { EArchivesCommand } from "@xrf/ipc/commands/archives";
import { mockInvoke, mockIpcCommands, resetMockInvoke } from "@xrf/ipc/testing";
import { EJobOutcome } from "@xrf/ipc/types/xrf-job";
import { ArchivePatchConfig, ArchivePatchResult } from "@xrf/ipc/types/xrf-pack";

import { mockInjectedService } from "@/fixtures/utils/container";
import { deferred } from "@/fixtures/utils/promise";
import { ArchivesPatcherService, EPatcherSection } from "@/tools/archives-patcher/services/archives-patcher.service";

/** A configuration shaped like the one the backend answers with. */
const CONFIG: ArchivePatchConfig = {
  input: "",
  target: null,
  destination: "",
  name: "patch",
  include: [],
  ignore: [],
  excludeExtensions: [],
  header: "[header]\r\nauto_load = true\r\nentry_point = $fs_root$\\gamedata\\\r\n",
  mode: "Compress",
  maxVolumeSize: 1900 * BYTES_PER_MEGABYTE,
  isWithOversizedVolumes: false,
  volumeExtension: "Db",
};

function mockPatcherService(config: ArchivePatchConfig = CONFIG): ArchivesPatcherService {
  const { service } = mockInjectedService(ArchivesPatcherService);

  service.config = { ...config };

  return service;
}

describe("ArchivesPatcherService configuration", () => {
  beforeEach(() => {
    resetMockInvoke();
  });

  it("opens on the configuration the backend owns", async () => {
    const defaults: ArchivePatchConfig = { ...CONFIG, name: "from-backend" };

    mockIpcCommands({ [EArchivesCommand.DEFAULT_PATCH_CONFIG]: defaults });

    const { service } = mockInjectedService(ArchivesPatcherService);

    await service.load();

    expect(service.config).toStrictEqual(defaults);
  });

  it("opens on the comparison section, which is the only field a run cannot infer", () => {
    expect(mockPatcherService().section).toBe(EPatcherSection.COMPARISON);
  });

  it("counts nothing as unsaved until a file has been read or written", () => {
    const service: ArchivesPatcherService = mockPatcherService();

    service.patchConfig({ include: ["configs"] });

    expect(service.isDirty).toBe(false);
  });

  it("counts an edit after an import as unsaved", async () => {
    mockIpcCommands({ [EArchivesCommand.IMPORT_PATCH_CONFIG]: { ...CONFIG, include: ["configs"] } });

    const service: ArchivesPatcherService = mockPatcherService();

    await service.importConfig("C:\\work\\patch.json");

    expect(service.isDirty).toBe(false);
    expect(service.configName).toBe("patch.json");

    service.patchConfig({ ignore: ["configs\\text"] });

    expect(service.isDirty).toBe(true);
  });

  it("does not count a path as an edit, since a file never carries one", async () => {
    // What keeps a freshly imported configuration from reading as unsaved the moment a game is chosen.
    mockIpcCommands({ [EArchivesCommand.IMPORT_PATCH_CONFIG]: CONFIG });

    const service: ArchivesPatcherService = mockPatcherService();

    await service.importConfig("C:\\work\\patch.json");

    service.patchConfig({ input: "C:\\Games\\Anomaly", destination: "C:\\out", name: "mypatch" });

    expect(service.isDirty).toBe(false);
  });

  it("sends the whole configuration when exporting, and remembers the file", async () => {
    mockIpcCommands({ [EArchivesCommand.EXPORT_PATCH_CONFIG]: null });

    const service: ArchivesPatcherService = mockPatcherService();

    service.patchConfig({ include: ["configs"] });

    await service.exportConfig("C:\\work\\patch.json");

    expect(mockInvoke).toHaveBeenCalledWith(
      EArchivesCommand.EXPORT_PATCH_CONFIG,
      expect.objectContaining({ path: "C:\\work\\patch.json" })
    );
    expect(service.configName).toBe("patch.json");
    expect(service.isDirty).toBe(false);
  });

  it("reports an import failure without losing the open configuration", async () => {
    mockIpcCommands({
      [EArchivesCommand.IMPORT_PATCH_CONFIG]: () => {
        throw new Error("not a patching configuration");
      },
    });

    const service: ArchivesPatcherService = mockPatcherService();

    await service.importConfig("C:\\work\\patch.json");

    expect(service.error).toContain("not a patching configuration");
    expect(service.config?.name).toBe("patch");
    expect(service.configPath).toBeNull();
  });
});

describe("ArchivesPatcherService error recovery", () => {
  beforeEach(() => {
    resetMockInvoke();
  });

  it("clears a configuration failure when a comparison succeeds", async () => {
    const result: ArchivePatchResult = {
      outcome: EJobOutcome.COMPLETED,
      added: [],
      modified: [],
      unchanged: 1,
      origins: [],
      payloadsRead: 0,
      sizeCarried: 0,
      publication: { kind: "compared" },
      duration: 0,
      compareDuration: 0,
      packDuration: 0,
    };

    mockIpcCommands({
      [EArchivesCommand.IMPORT_PATCH_CONFIG]: () => {
        throw new Error("invalid configuration");
      },
      [EArchivesCommand.COMPARE_ARCHIVES]: result,
    });

    const service: ArchivesPatcherService = mockPatcherService();

    await service.importConfig("C:\\work\\patch.json");

    expect(service.error).toBe("invalid configuration");

    await service.compare({ config: CONFIG, isForced: false, isVerifyingPayload: false });

    expect(service.operation.result).toEqual(result);
    expect(service.error).toBeNull();
  });

  it("clears a job failure when configuration export succeeds", async () => {
    mockIpcCommands({
      [EArchivesCommand.COMPARE_ARCHIVES]: () => {
        throw new Error("cannot read input");
      },
      [EArchivesCommand.EXPORT_PATCH_CONFIG]: null,
    });

    const service: ArchivesPatcherService = mockPatcherService();

    await service.compare({ config: CONFIG, isForced: false, isVerifyingPayload: false });

    expect(service.operation.error).toBe("cannot read input");
    expect(service.error).toBe("cannot read input");

    await service.exportConfig("C:\\work\\patch.json");

    expect(service.configPath).toBe("C:\\work\\patch.json");
    expect(service.operation.error).toBeNull();
    expect(service.error).toBeNull();
  });
});

describe("ArchivesPatcherService destination check", () => {
  beforeEach(() => {
    resetMockInvoke();
  });

  it("lists existing volumes when the destination check is called", async () => {
    const config: ArchivePatchConfig = { ...CONFIG, destination: "C:\\out" };
    const volumes: Array<string> = ["C:\\out\\patch.db0"];

    mockIpcCommands({ [EArchivesCommand.LIST_PATCH_VOLUMES]: volumes });

    const service: ArchivesPatcherService = mockPatcherService();

    await service.checkDestination(config);

    expect(mockInvoke).toHaveBeenCalledWith(EArchivesCommand.LIST_PATCH_VOLUMES, { config });
    expect(service.publishedVolumes).toEqual(volumes);
  });

  it("ignores an older destination listing that finishes after its replacement", async () => {
    const olderResponse = deferred<Array<string>>();

    mockIpcCommands({ [EArchivesCommand.LIST_PATCH_VOLUMES]: () => olderResponse.promise });

    const service: ArchivesPatcherService = mockPatcherService();
    const olderCheck = flowResult(service.checkDestination({ ...CONFIG, destination: "C:\\old" }));
    const newerVolumes: Array<string> = ["C:\\new\\patch.db0"];

    mockIpcCommands({ [EArchivesCommand.LIST_PATCH_VOLUMES]: newerVolumes });

    await service.checkDestination({ ...CONFIG, destination: "C:\\new" });

    expect(service.publishedVolumes).toEqual(newerVolumes);

    olderResponse.resolve(["C:\\old\\patch.db0"]);
    await olderResponse.promise;
    await olderCheck;

    expect(service.publishedVolumes).toEqual(newerVolumes);
  });
});

describe("ArchivesPatcherService volume size", () => {
  beforeEach(() => {
    resetMockInvoke();
  });

  it("allows increasing the volume size after lowering it", () => {
    const service: ArchivesPatcherService = mockPatcherService();

    service.setVolumeSize("512");
    service.setVolumeSize("1024");

    expect(service.volumeSizeError).toBeNull();
    expect(service.volumeSizeBytes).toBe(1024 * BYTES_PER_MEGABYTE);
    expect(service.maxVolumeSizeMegabytes).toBe(1900);
  });

  it("restores the default volume size when an override is cleared", () => {
    const service: ArchivesPatcherService = mockPatcherService();

    service.setVolumeSize("512");
    service.setVolumeSize("");

    expect(service.config?.maxVolumeSize).toBe(CONFIG.maxVolumeSize);
    expect(service.volumeSizeBytes).toBe(CONFIG.maxVolumeSize);
    expect(service.volumeSizeError).toBeNull();
  });

  it("resolves a usable ceiling without changing the configuration", () => {
    const service: ArchivesPatcherService = mockPatcherService();

    service.setVolumeSize("512");

    expect(service.volumeSizeError).toBeNull();
    expect(service.volumeSizeBytes).toBe(512 * BYTES_PER_MEGABYTE);
    expect(service.config?.maxVolumeSize).toBe(CONFIG.maxVolumeSize);
  });

  it("reports an unusable ceiling rather than writing it", () => {
    const service: ArchivesPatcherService = mockPatcherService();

    service.setVolumeSize("9000");

    expect(service.volumeSizeError).toContain("between 1 and");
    expect(service.config?.maxVolumeSize).toBe(1900 * BYTES_PER_MEGABYTE);
    expect(service.volumeSizeBytes).toBe(CONFIG.maxVolumeSize);
  });

  it("treats an empty ceiling as the format's own", () => {
    const service: ArchivesPatcherService = mockPatcherService();

    service.setVolumeSize("");

    expect(service.volumeSizeError).toBeNull();
    expect(service.volumeSizeBytes).toBe(CONFIG.maxVolumeSize);
  });
});
