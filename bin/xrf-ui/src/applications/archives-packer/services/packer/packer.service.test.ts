import { beforeEach, describe, expect, it } from "@jest/globals";
import { EventBus } from "@wirestate/core";

import { FALLBACK_PACK_CONFIG } from "@/applications/archives-packer/lib/pack-config";
import { PackerService } from "@/applications/archives-packer/services/packer/index";
import { ArchivePackConfig, ArchivePackResult } from "@/core/bindings/types/xrf-pack";
import { IJobSettledPayload, JOB_SETTLED_EVENT } from "@/core/jobs/lib";
import { mockInvoke, resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { IInjectedServiceMockDescriptor, mockInjectedService } from "@/fixtures/utils/container";
import { BYTES_PER_MEGABYTE } from "@/lib/memory/size";

function mockPackerService(config: ArchivePackConfig = FALLBACK_PACK_CONFIG): PackerService {
  const { service } = mockInjectedService(PackerService);

  service.config = config;

  return service;
}

describe("PackerService defaults", () => {
  beforeEach(() => {
    resetMockInvoke();
  });

  it("opens on the configuration the packer owns", async () => {
    const defaults: ArchivePackConfig = { ...FALLBACK_PACK_CONFIG, name: "from-packer" };

    setMockInvokeResponses({ ["plugin:archives|default_pack_config"]: defaults });

    const { service } = mockInjectedService(PackerService);

    await service.onProvision();

    expect(service.config).toStrictEqual(defaults);
  });

  it("opens an editable configuration even when the packer cannot answer", async () => {
    setMockInvokeResponses({
      ["plugin:archives|default_pack_config"]: () => {
        throw new Error("no backend");
      },
    });

    const { service } = mockInjectedService(PackerService);

    await service.onProvision();

    expect(service.config).toStrictEqual(FALLBACK_PACK_CONFIG);
  });
});

describe("PackerService editing", () => {
  beforeEach(() => {
    resetMockInvoke();
  });

  it("drops the last result when the configuration it described changes", async () => {
    const service: PackerService = mockPackerService();

    setMockInvokeResponses({
      ["plugin:archives|pack_directory"]: { volumes: ["gamedata.db"], filesTotal: 1 },
    });

    await service.pack({ ...FALLBACK_PACK_CONFIG, source: "C:\\in", destination: "C:\\out" }, false);

    expect(service.result).not.toBeNull();

    service.patchConfig({ name: "renamed" });

    expect(service.result).toBeNull();
  });

  it("keeps a failed run reportable instead of throwing at the editor", async () => {
    const service: PackerService = mockPackerService();

    setMockInvokeResponses({
      ["plugin:archives|pack_directory"]: () => {
        throw new Error("source is empty");
      },
    });

    await service.pack({ ...FALLBACK_PACK_CONFIG, source: "C:\\in", destination: "C:\\out" }, false);

    expect(service.error).toBe("source is empty");
    expect(service.result).toBeNull();
    expect(service.isBusy).toBe(false);
  });
});

describe("PackerService volume ceiling", () => {
  it("packs with the packer's own maximum while nothing is typed", () => {
    const service: PackerService = mockPackerService();

    expect(service.volumeSizeError).toBeNull();
    expect(service.volumeSizeBytes).toBe(FALLBACK_PACK_CONFIG.maxVolumeSize);
  });

  it("refuses a ceiling the engine would not open", () => {
    const service: PackerService = mockPackerService();

    service.setVolumeSize(String(service.maxVolumeSizeMegabytes + 1));

    expect(service.volumeSizeError).not.toBeNull();
    // Still the configured ceiling, so a rejected value is never the one that gets packed.
    expect(service.volumeSizeBytes).toBe(FALLBACK_PACK_CONFIG.maxVolumeSize);
  });

  it("packs with a usable typed ceiling", () => {
    const service: PackerService = mockPackerService();

    service.setVolumeSize("512");

    expect(service.volumeSizeError).toBeNull();
    expect(service.volumeSizeBytes).toBe(512 * BYTES_PER_MEGABYTE);
  });
});

describe("PackerService configuration files", () => {
  beforeEach(() => {
    resetMockInvoke();
  });

  it("is not unsaved before a file is involved", () => {
    const service: PackerService = mockPackerService();

    service.patchConfig({ excludeExtensions: [".thm"] });

    expect(service.isDirty).toBe(false);
    expect(service.configName).toBeNull();
  });

  it("counts edits made after an import as unsaved", async () => {
    const imported: ArchivePackConfig = { ...FALLBACK_PACK_CONFIG, excludeExtensions: [".thm"] };

    setMockInvokeResponses({ ["plugin:archives|import_pack_config"]: imported });

    const service: PackerService = mockPackerService();

    await service.importConfig("C:\\configs\\pack.ltx");

    expect(service.isDirty).toBe(false);
    expect(service.configName).toBe("pack.ltx");

    service.patchConfig({ excludeExtensions: [".thm", ".tga"] });

    expect(service.isDirty).toBe(true);
  });

  it("does not count the paths a run chooses as unsaved edits", async () => {
    setMockInvokeResponses({ ["plugin:archives|import_pack_config"]: FALLBACK_PACK_CONFIG });

    const service: PackerService = mockPackerService();

    await service.importConfig("C:\\configs\\pack.ltx");

    service.patchConfig({ name: "renamed", source: "C:\\in", destination: "C:\\out" });

    expect(service.isDirty).toBe(false);
  });

  it("settles the configuration once it is written back", async () => {
    setMockInvokeResponses({ ["plugin:archives|import_pack_config"]: FALLBACK_PACK_CONFIG });

    const service: PackerService = mockPackerService();

    await service.importConfig("C:\\configs\\pack.ltx");

    service.patchConfig({ excludeExtensions: [".thm"] });

    expect(service.isDirty).toBe(true);

    await service.exportConfig("C:\\configs\\other.ltx");

    expect(service.isDirty).toBe(false);
    expect(service.configName).toBe("other.ltx");
    expect(mockInvoke).toHaveBeenCalledWith("plugin:archives|export_pack_config", {
      path: "C:\\configs\\other.ltx",
      config: service.config,
    });
  });

  // The two serializations carry the same payload, so nothing here may depend on which one a path names: the backend
  // picks the codec from the extension, and this service only ever holds a path.
  it.each(["ltx", "json"])("tracks the same unsaved state whichever format a file names (%s)", async (extension) => {
    const imported: ArchivePackConfig = { ...FALLBACK_PACK_CONFIG, excludeExtensions: [".thm"] };

    setMockInvokeResponses({ ["plugin:archives|import_pack_config"]: imported });

    const service: PackerService = mockPackerService();

    await service.importConfig(`C:\\configs\\pack.${extension}`);

    expect(service.config).toStrictEqual(imported);
    expect(service.configName).toBe(`pack.${extension}`);
    expect(service.isDirty).toBe(false);

    service.patchConfig({ excludeExtensions: [".thm", ".tga"] });

    expect(service.isDirty).toBe(true);

    await service.exportConfig(`C:\\configs\\other.${extension}`);

    expect(service.isDirty).toBe(false);
    expect(service.configName).toBe(`other.${extension}`);
    expect(mockInvoke).toHaveBeenCalledWith("plugin:archives|export_pack_config", {
      path: `C:\\configs\\other.${extension}`,
      config: service.config,
    });
  });

  it("reports what the backend refused about a configuration file it cannot read", async () => {
    // An unsupported extension is the backend's refusal, not this service's rule; it only has to surface it.
    setMockInvokeResponses({
      ["plugin:archives|import_pack_config"]: () => {
        throw new Error("'.txt' is not a packing configuration");
      },
    });

    const service: PackerService = mockPackerService();

    await service.importConfig("C:\\configs\\pack.txt");

    expect(service.error).toContain(".txt");
    expect(service.configPath).toBeNull();
  });
});

describe("PackerService adoption", () => {
  beforeEach(() => {
    resetMockInvoke();
  });

  /**
   * A packer wired the way the application wires one, with its event handlers live.
   *
   * Provisioned rather than merely resolved, because what is under test here arrives on the event bus: a service that
   * was never provisioned is subscribed to nothing and would pass every assertion by doing nothing at all.
   *
   * @param listed - What the jobs listing answers, which is how a pack is already running before this window loads.
   * @returns The packer and its container.
   */
  function provisionedPacker(listed: Array<unknown> = []): IInjectedServiceMockDescriptor<PackerService> {
    setMockInvokeResponses({
      ["plugin:jobs|list"]: listed,
      ["plugin:jobs|attach"]: true,
      ["plugin:archives|default_pack_config"]: FALLBACK_PACK_CONFIG,
    });

    const descriptor: IInjectedServiceMockDescriptor<PackerService> = mockInjectedService(PackerService);

    descriptor.container.provision();

    return descriptor;
  }

  it("renders the answer of a pack it never awaited", () => {
    // The command replied to a page that no longer exists. What the backend retained arrives as an announcement, and
    // the cast to a pack result belongs here because this is what knows a pack when it sees one.
    const { service, container }: IInjectedServiceMockDescriptor<PackerService> = provisionedPacker();
    const result: Partial<ArchivePackResult> = { volumes: ["textures.db0"], filesTotal: 2 };

    container.get(EventBus).emit<IJobSettledPayload>(JOB_SETTLED_EVENT, {
      id: "b8f0",
      kind: "archives.pack",
      conclusion: "completed",
      error: null,
      result,
    });

    expect(service.result).toStrictEqual(result);
  });

  it("leaves another tool's job alone", () => {
    const { service, container }: IInjectedServiceMockDescriptor<PackerService> = provisionedPacker();

    container.get(EventBus).emit<IJobSettledPayload>(JOB_SETTLED_EVENT, {
      id: "b8f0",
      kind: "archives.unpack",
      conclusion: "completed",
      error: null,
      result: { extracted: 4 },
    });

    expect(service.result).toBeNull();
  });

  it("shows why a pack it never awaited failed", () => {
    const { service, container }: IInjectedServiceMockDescriptor<PackerService> = provisionedPacker();

    container.get(EventBus).emit<IJobSettledPayload>(JOB_SETTLED_EVENT, {
      id: "b8f0",
      kind: "archives.pack",
      conclusion: "failed",
      error: "volume cap refuses particles.xr",
      result: null,
    });

    expect(service.result).toBeNull();
    expect(service.error).toBe("volume cap refuses particles.xr");
  });
});
