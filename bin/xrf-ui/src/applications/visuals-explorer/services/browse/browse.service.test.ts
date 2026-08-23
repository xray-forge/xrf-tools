import { beforeEach, describe, expect, it } from "@jest/globals";
import { isComputedProp, isObservableProp } from "@wirestate/mobx";

import { VisualsBrowseService } from "@/applications/visuals-explorer/services/browse/index";
import { XrayAsset } from "@/core/bindings/types/xrf-vfs";
import { resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { Nullable } from "@/lib/types/general";

function mockVisual(logicalPath: string): XrayAsset {
  return {
    container: { kind: "directory", relativePath: logicalPath, root: "C:\\gamedata" },
    logicalPath,
  };
}

describe("VisualsBrowseService", () => {
  beforeEach(() => {
    resetMockInvoke();
  });

  it("applies its mobx annotations", () => {
    const { service } = mockInjectedService(VisualsBrowseService);

    expect(isObservableProp(service, "world")).toBe(true);
    expect(isObservableProp(service, "visuals")).toBe(true);
    expect(isComputedProp(service, "isBrowsing")).toBe(true);
    expect(isComputedProp(service, "root")).toBe(true);
    expect(isComputedProp(service, "roots")).toBe(true);
  });

  it("lists every visual of the root it browses, asking for models only", async () => {
    const { service } = mockInjectedService(VisualsBrowseService);

    let listParameters: Nullable<Record<string, unknown>> = null;

    setMockInvokeResponses({
      ["plugin:assets|list_assets"]: (parameters?: Record<string, unknown>) => {
        listParameters = parameters ?? null;

        return [mockVisual("meshes\\wpn\\wpn_ak74.ogf")];
      },
    });

    await service.openRoot("C:\\gamedata");

    // No subject asset: a browsed root is the world itself, not a neighbourhood around one model.
    expect(listParameters).toEqual({ kind: "ogf", world: { asset: null, roots: ["C:\\gamedata"] } });
    expect(service.visuals.value).toHaveLength(1);
    expect(service.isBrowsing).toBe(true);
    expect(service.roots).toEqual(["C:\\gamedata"]);
  });

  it("reports a failed listing without pretending the root is empty of intent", async () => {
    const { service } = mockInjectedService(VisualsBrowseService);

    setMockInvokeResponses({
      ["plugin:assets|list_assets"]: () => {
        throw new Error("root does not exist");
      },
    });

    await service.openRoot("C:\\missing");

    expect(service.visuals.error?.message).toBe("root does not exist");
    expect(service.visuals.isLoading).toBe(false);
    expect(service.isBrowsing).toBe(true);
  });

  it("records the browsed world in the backend rather than keeping it to itself", async () => {
    let recorded: Nullable<Record<string, unknown>> = null;

    setMockInvokeResponses({
      ["plugin:assets|list_assets"]: [mockVisual("meshes\\actors\\stalker.ogf")],
      ["plugin:visuals|open_browse"]: (parameters?: Record<string, unknown>) => {
        recorded = parameters ?? null;

        return null;
      },
    });

    const { service } = mockInjectedService(VisualsBrowseService);

    await service.openRoot("C:\\gamedata");

    expect(recorded).toEqual({ world: { asset: null, roots: ["C:\\gamedata"] } });
  });

  it("comes back to the world the backend is still browsing after a reload", async () => {
    // The session lives where every other application's does; a reload asks for it and derives the listing again, which
    // is cheap because the mounts that listing reads are already cached.
    setMockInvokeResponses({
      ["plugin:visuals|get_browse"]: { asset: null, roots: ["C:\\gamedata"] },
      ["plugin:assets|list_assets"]: [mockVisual("meshes\\actors\\stalker.ogf")],
    });

    const { service } = mockInjectedService(VisualsBrowseService);

    await service.onProvision();

    expect(service.root).toBe("C:\\gamedata");
    expect(service.visuals.value).toHaveLength(1);
  });

  it("releases the browsed world on deactivation, so leaving closes in place", async () => {
    // The selection is dropped on the way out, so a session left open would come back beside a model the backend no
    // longer has. A reload runs no deactivation, which is what keeps the restore above working.
    const released: Array<string> = [];

    setMockInvokeResponses({
      ["plugin:assets|list_assets"]: [mockVisual("meshes\\actors\\stalker.ogf")],
      ["plugin:visuals|open_browse"]: null,
      ["plugin:visuals|close_browse"]: () => {
        released.push("closed");

        return null;
      },
    });

    const { service } = mockInjectedService(VisualsBrowseService);

    await service.openRoot("C:\\gamedata");

    service.onDeactivation();

    expect(service.root).toBeNull();
    expect(service.visuals.value).toEqual([]);
    expect(released).toEqual(["closed"]);
  });

  it("forgets the world when browsing is closed, backend included", async () => {
    const closed: Array<string> = [];

    setMockInvokeResponses({
      ["plugin:assets|list_assets"]: [],
      ["plugin:visuals|open_browse"]: null,
      ["plugin:visuals|close_browse"]: () => {
        closed.push("closed");

        return null;
      },
      // A session the backend no longer holds is what a later provisioning must find.
      ["plugin:visuals|get_browse"]: null,
    });

    const { service } = mockInjectedService(VisualsBrowseService);

    await service.openRoot("C:\\gamedata");
    await service.close();

    expect(service.root).toBeNull();
    expect(service.isBrowsing).toBe(false);
    expect(service.roots).toEqual([]);
    expect(closed).toEqual(["closed"]);

    const next = mockInjectedService(VisualsBrowseService);

    await next.service.onProvision();

    expect(next.service.root).toBeNull();
  });
});
