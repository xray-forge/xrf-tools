import { beforeEach, describe, expect, it } from "@jest/globals";

import { EPatcherSection, PatcherService } from "@/applications/archives-patcher/services/patcher/index";
import { ArchivePatchConfig } from "@/core/ipc/types/xrf-pack";
import { mockInvoke, resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { BYTES_PER_MEGABYTE } from "@/lib/memory/size";

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

function mockPatcherService(config: ArchivePatchConfig = CONFIG): PatcherService {
  const { service } = mockInjectedService(PatcherService);

  service.config = { ...config };

  return service;
}

describe("PatcherService configuration", () => {
  beforeEach(() => {
    resetMockInvoke();
  });

  it("opens on the configuration the backend owns", async () => {
    const defaults: ArchivePatchConfig = { ...CONFIG, name: "from-backend" };

    setMockInvokeResponses({ ["plugin:archives|default_patch_config"]: defaults });

    const { service } = mockInjectedService(PatcherService);

    await service.load();

    expect(service.config).toStrictEqual(defaults);
  });

  it("opens on the comparison section, which is the only field a run cannot infer", () => {
    expect(mockPatcherService().section).toBe(EPatcherSection.COMPARISON);
  });

  it("counts nothing as unsaved until a file has been read or written", () => {
    const service: PatcherService = mockPatcherService();

    service.patchConfig({ include: ["configs"] });

    expect(service.isDirty).toBe(false);
  });

  it("counts an edit after an import as unsaved", async () => {
    setMockInvokeResponses({ ["plugin:archives|import_patch_config"]: { ...CONFIG, include: ["configs"] } });

    const service: PatcherService = mockPatcherService();

    await service.importConfig("C:\\work\\patch.json");

    expect(service.isDirty).toBe(false);
    expect(service.configName).toBe("patch.json");

    service.patchConfig({ ignore: ["configs\\text"] });

    expect(service.isDirty).toBe(true);
  });

  it("does not count a path as an edit, since a file never carries one", async () => {
    // What keeps a freshly imported configuration from reading as unsaved the moment a game is chosen.
    setMockInvokeResponses({ ["plugin:archives|import_patch_config"]: CONFIG });

    const service: PatcherService = mockPatcherService();

    await service.importConfig("C:\\work\\patch.json");

    service.patchConfig({ input: "C:\\Games\\Anomaly", destination: "C:\\out", name: "mypatch" });

    expect(service.isDirty).toBe(false);
  });

  it("sends the whole configuration when exporting, and remembers the file", async () => {
    setMockInvokeResponses({ ["plugin:archives|export_patch_config"]: null });

    const service: PatcherService = mockPatcherService();

    service.patchConfig({ include: ["configs"] });

    await service.exportConfig("C:\\work\\patch.json");

    expect(mockInvoke).toHaveBeenCalledWith(
      "plugin:archives|export_patch_config",
      expect.objectContaining({ path: "C:\\work\\patch.json" })
    );
    expect(service.configName).toBe("patch.json");
    expect(service.isDirty).toBe(false);
  });

  it("reports an import failure without losing the open configuration", async () => {
    setMockInvokeResponses({
      ["plugin:archives|import_patch_config"]: () => {
        throw new Error("not a patching configuration");
      },
    });

    const service: PatcherService = mockPatcherService();

    await service.importConfig("C:\\work\\patch.json");

    expect(service.error).toContain("not a patching configuration");
    expect(service.config?.name).toBe("patch");
    expect(service.configPath).toBeNull();
  });
});

describe("PatcherService volume size", () => {
  beforeEach(() => {
    resetMockInvoke();
  });

  it("writes a usable ceiling into the configuration", () => {
    const service: PatcherService = mockPatcherService();

    service.setVolumeSize("512");

    expect(service.volumeSizeError).toBeNull();
    expect(service.config?.maxVolumeSize).toBe(512 * BYTES_PER_MEGABYTE);
  });

  it("reports an unusable ceiling rather than writing it", () => {
    const service: PatcherService = mockPatcherService();

    service.setVolumeSize("9000");

    expect(service.volumeSizeError).toContain("between 1 and");
    expect(service.config?.maxVolumeSize).toBe(1900 * BYTES_PER_MEGABYTE);
  });

  it("treats an empty ceiling as the format's own", () => {
    const service: PatcherService = mockPatcherService();

    service.setVolumeSize("");

    expect(service.volumeSizeError).toBeNull();
  });
});
