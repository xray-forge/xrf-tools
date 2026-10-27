import { afterEach, describe, expect, it, jest } from "@jest/globals";
import { flowResult } from "@wirestate/mobx";
import { EAssetsBulkRoute } from "@xrf/ipc/commands/assets-bulk";
import { texturesCommands } from "@xrf/ipc/commands/textures";
import { IpcBulkRouteFetch, mockIpcBulkRoutes } from "@xrf/ipc/testing";
import { ETextureSource, TextureDescription } from "@xrf/ipc/types/xrf-app-textures";
import { XrayRoots } from "@xrf/ipc/types/xrf-vfs";

import { createRoots } from "@/core/assets/lib";
import { mockTextureDescription } from "@/core/textures/fixtures/texture.mocks";
import { TextureSelectionService } from "@/core/textures/services/selection";
import { mockInjectedService } from "@/fixtures/utils/container";
import { deferred } from "@/fixtures/utils/promise";
import { noop } from "@/lib/callbacks/noop";

const TEXTURE: string = "C:\\mods\\mine\\gamedata\\textures\\wall.dds";
const INSTALLATION: string = "C:\\Games\\stalker";

/**
 * Creates a selection service with successful description and preview responses.
 *
 * @returns The service, native command spies, and their response fixtures.
 */
function mockService() {
  const { service } = mockInjectedService(TextureSelectionService);
  const description = mockTextureDescription();
  const preview = new ArrayBuffer(8);
  const describeTexture = jest.spyOn(texturesCommands, "describe").mockResolvedValue(description);
  const readTexture = jest.fn<IpcBulkRouteFetch<EAssetsBulkRoute.READ_TEXTURE>>().mockResolvedValue(preview);

  mockIpcBulkRoutes({ [EAssetsBulkRoute.READ_TEXTURE]: readTexture });

  return { service, description, preview, describeTexture, readTexture };
}

describe("TextureSelectionService", () => {
  afterEach(() => {
    jest.restoreAllMocks();
  });

  it("resolves a loose file in its own neighbourhood when no further tree was named", async () => {
    const { describeTexture, service } = mockService();

    await service.openFile(TEXTURE);

    // Centred on the file, and nothing behind it: with no ambient set anywhere, reading only what you opened is the
    // honest default rather than a silent failure to layer.
    expect(describeTexture).toHaveBeenCalledWith({ kind: "file", path: TEXTURE }, createRoots([], TEXTURE));
  });

  it("resolves a loose file against the tree the surface was opened with", async () => {
    const { describeTexture, service } = mockService();

    service.setAssetRoot(INSTALLATION);

    await service.openFile(TEXTURE);

    // What replaced the configured game data: a mod tree's texture still finds the bump pair the base game holds,
    // because the open said where to look rather than a setting somewhere else.
    expect(describeTexture).toHaveBeenCalledWith({ kind: "file", path: TEXTURE }, createRoots([INSTALLATION], TEXTURE));
  });

  it("leaves a listing's own roots alone, because a browsed row is already addressed", async () => {
    const { describeTexture, service } = mockService();
    const browsed: XrayRoots = createRoots(["C:\\mods\\mine\\gamedata", INSTALLATION]);

    service.setAssetRoot(INSTALLATION);

    await service.open({ kind: ETextureSource.ASSET, reference: "textures\\wall" }, browsed);

    expect(describeTexture).toHaveBeenCalledWith({ kind: "asset", reference: "textures\\wall" }, browsed);
  });

  it("retries a failed asset description with the original source and roots", async () => {
    const { service, description, preview, describeTexture } = mockService();
    const roots = createRoots([INSTALLATION]);
    const source = { kind: "asset" as const, reference: "textures\\wall" };

    describeTexture.mockRejectedValueOnce(new Error("Cannot describe texture"));

    await service.open(source, roots);

    expect(service.selected.error?.message).toBe("Cannot describe texture");
    expect(service.roots).toBeNull();

    await service.retry();

    expect(describeTexture.mock.calls).toEqual([
      [source, roots],
      [source, roots],
    ]);
    expect(service.selected.value).toEqual(description);
    expect(service.selected.error).toBeNull();
    expect(service.preview.value).toBe(preview);
  });

  it("retries a loose file with its original roots after the configured root changes", async () => {
    const { service, describeTexture } = mockService();
    const roots = createRoots([INSTALLATION], TEXTURE);

    describeTexture.mockRejectedValueOnce(new Error("Cannot describe texture"));
    service.setAssetRoot(INSTALLATION);

    await service.openFile(TEXTURE);

    service.setAssetRoot("C:\\Games\\replacement");

    await service.retry();

    expect(describeTexture.mock.calls).toEqual([
      [{ kind: "file", path: TEXTURE }, roots],
      [{ kind: "file", path: TEXTURE }, roots],
    ]);
    expect(service.selected.error).toBeNull();
  });

  it("retains the description after a preview failure and retries the complete request", async () => {
    const { service, description, preview, describeTexture, readTexture } = mockService();
    const roots = createRoots([INSTALLATION]);

    readTexture.mockRejectedValueOnce(new Error("Cannot decode texture"));

    await service.open({ kind: ETextureSource.ASSET, reference: "textures\\wall" }, roots);

    expect(service.selected.value).toEqual(description);
    expect(service.selected.error).toBeNull();
    expect(service.preview.error?.message).toBe("Cannot decode texture");

    await service.retry();

    expect(describeTexture.mock.calls).toEqual([
      [{ kind: "asset", reference: "textures\\wall" }, roots],
      [{ kind: "asset", reference: "textures\\wall" }, roots],
    ]);
    expect(readTexture).toHaveBeenCalledTimes(2);
    expect(service.preview.value).toBe(preview);
    expect(service.preview.error).toBeNull();
  });

  it("discards a description received after clearing without starting its preview", async () => {
    const { service, description, describeTexture, readTexture } = mockService();
    const describing = deferred<TextureDescription>();

    describeTexture.mockReturnValueOnce(describing.promise);

    const loading = flowResult(service.openFile(TEXTURE));

    expect(describeTexture).toHaveBeenCalledTimes(1);

    service.clear();
    describing.resolve(description);

    await describing.promise;
    await loading;

    expect(service.selected.value).toBeNull();
    expect(service.selected.isLoading).toBe(false);
    expect(service.preview.value).toBeNull();
    expect(readTexture).not.toHaveBeenCalled();
  });

  it("discards preview bytes received after clearing the selected texture", async () => {
    const { service, preview, readTexture } = mockService();
    const reading = deferred<ArrayBuffer>();
    const started = deferred<void>();

    readTexture.mockImplementationOnce(() => {
      started.resolve();

      return reading.promise;
    });

    const loading = flowResult(service.openFile(TEXTURE));

    await started.promise;

    service.clear();
    reading.resolve(preview);

    await reading.promise;
    await loading;

    expect(service.selected.value).toBeNull();
    expect(service.preview.value).toBeNull();
    expect(service.preview.isLoading).toBe(false);
  });

  it("discards a description failure received after deactivation", async () => {
    const { service, describeTexture, readTexture } = mockService();
    const describing = deferred<TextureDescription>();

    describeTexture.mockReturnValueOnce(describing.promise);

    const loading = flowResult(service.openFile(TEXTURE));

    service.onDeactivation();
    describing.reject(new Error("Previous texture unavailable"));

    await describing.promise.catch(noop);
    await loading;

    expect(service.selected.value).toBeNull();
    expect(service.selected.error).toBeNull();
    expect(service.preview.error).toBeNull();
    expect(readTexture).not.toHaveBeenCalled();
  });

  it("keeps the replacement texture when the previous preview arrives late", async () => {
    const { service, describeTexture, readTexture } = mockService();
    const current = mockTextureDescription("textures\\replacement");
    const currentPreview = new ArrayBuffer(16);
    const reading = deferred<ArrayBuffer>();
    const started = deferred<void>();

    readTexture.mockImplementationOnce(() => {
      started.resolve();

      return reading.promise;
    });

    const previous = flowResult(service.openFile(TEXTURE));

    await started.promise;

    describeTexture.mockResolvedValueOnce(current);
    readTexture.mockResolvedValueOnce(currentPreview);

    await service.open({ kind: ETextureSource.ASSET, reference: current.reference }, current.roots);

    reading.resolve(new ArrayBuffer(4));

    await reading.promise;
    await previous;

    expect(service.selected.value).toEqual(current);
    expect(service.preview.value).toBe(currentPreview);
  });

  it("forgets the retry request when the selection is cleared", async () => {
    const { service, describeTexture, readTexture } = mockService();

    await service.openFile(TEXTURE);

    service.clear();

    await service.retry();

    expect(describeTexture).toHaveBeenCalledTimes(1);
    expect(readTexture).toHaveBeenCalledTimes(1);
    expect(service.selected.value).toBeNull();
    expect(service.preview.value).toBeNull();
  });
});
