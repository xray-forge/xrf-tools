import { beforeEach, describe, expect, it } from "@jest/globals";
import { isComputedProp } from "@wirestate/mobx";
import { EAssetsCommand } from "@xrf/ipc/commands/assets";
import { ESessionsCommand } from "@xrf/ipc/commands/sessions";
import { EVisualsCommand } from "@xrf/ipc/commands/visuals";
import { SessionScopeService } from "@xrf/ipc/session";
import { mockIpcCommands, resetMockInvoke } from "@xrf/ipc/testing";
import { XrayAsset, XrayRoots } from "@xrf/ipc/types/xrf-vfs";
import { Nullable } from "@xrf/types";

import { VisualsBrowseService } from "@/applications/visuals-explorer/services/visuals-browse";
import { createRoots } from "@/core/assets/lib";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";

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

    expect(isComputedProp(service, "isBrowsing")).toBe(true);
    expect(isComputedProp(service, "rootsLabel")).toBe(true);
    expect(isComputedProp(service, "rootPaths")).toBe(true);
  });

  it("lists every visual of the root it browses, asking for models only", async () => {
    const { service } = mockInjectedService(VisualsBrowseService);

    let listParameters: Nullable<Record<string, unknown>> = null;

    mockIpcCommands({
      [EVisualsCommand.OPEN_BROWSE]: mockSessionResponse(({ roots }: { roots: XrayRoots }) => roots),
      [EAssetsCommand.LIST_ASSETS]: (parameters?: Record<string, unknown>) => {
        listParameters = parameters ?? null;

        return [mockVisual("meshes\\wpn\\wpn_ak74.ogf")];
      },
    });

    await service.openRoot("C:\\gamedata");

    // No subject asset: a browsed root is the roots itself, not a neighbourhood around one model.
    expect(listParameters).toEqual({ kind: "ogf", roots: createRoots(["C:\\gamedata"]) });
    expect(service.visuals.value ?? []).toHaveLength(1);
    expect(service.isBrowsing).toBe(true);
    expect(service.rootPaths).toEqual(["C:\\gamedata"]);
  });

  it("reports a failed listing without pretending the root is empty of intent", async () => {
    const { service } = mockInjectedService(VisualsBrowseService);

    mockIpcCommands({
      [EVisualsCommand.OPEN_BROWSE]: mockSessionResponse(({ roots }: { roots: XrayRoots }) => roots),
      [EAssetsCommand.LIST_ASSETS]: () => {
        throw new Error("root does not exist");
      },
    });

    await expect(service.openRoot("C:\\missing")).resolves.toBe(true);

    expect(service.visuals.error?.message).toBe("root does not exist");
    expect(service.visuals.isLoading).toBe(false);
    expect(service.isBrowsing).toBe(true);
  });

  it("lists a new root from nothing, never showing the last root's visuals where its listing fails", async () => {
    const { service } = mockInjectedService(VisualsBrowseService);

    mockIpcCommands({
      [EVisualsCommand.OPEN_BROWSE]: mockSessionResponse(({ roots }: { roots: XrayRoots }) => roots),
      [EAssetsCommand.LIST_ASSETS]: [mockVisual("meshes\\actors\\stalker.ogf")],
    });

    await service.openRoot("C:\\gamedata");

    mockIpcCommands({
      [EVisualsCommand.OPEN_BROWSE]: mockSessionResponse(({ roots }: { roots: XrayRoots }) => roots),
      [EAssetsCommand.LIST_ASSETS]: () => {
        throw new Error("root does not exist");
      },
    });

    await service.openRoot("C:\\missing");

    expect(service.rootPaths).toEqual(["C:\\missing"]);
    expect(service.visuals.value).toBeNull();
  });

  it("reports a root that did not open, keeping what was browsed", async () => {
    const { service } = mockInjectedService(VisualsBrowseService);

    mockIpcCommands({
      [EVisualsCommand.OPEN_BROWSE]: mockSessionResponse(({ roots }: { roots: XrayRoots }) => roots),
      [EAssetsCommand.LIST_ASSETS]: [],
    });

    await service.openRoot("C:\\gamedata");

    mockIpcCommands({
      [EVisualsCommand.OPEN_BROWSE]: () => {
        throw new Error("not a directory");
      },
    });

    await expect(service.openRoot("C:\\missing")).resolves.toBe(false);

    expect(service.rootPaths).toEqual(["C:\\gamedata"]);
    expect(service.roots.error?.message).toBe("not a directory");
  });

  it("records the browsed roots in the backend rather than keeping it to itself", async () => {
    let recorded: Nullable<Record<string, unknown>> = null;

    mockIpcCommands({
      [EAssetsCommand.LIST_ASSETS]: [mockVisual("meshes\\actors\\stalker.ogf")],
      [EVisualsCommand.OPEN_BROWSE]: mockSessionResponse((parameters: { roots: XrayRoots }) => {
        recorded = parameters;

        return parameters.roots;
      }),
    });

    const { service } = mockInjectedService(VisualsBrowseService);

    await service.openRoot("C:\\gamedata");

    expect(recorded).toEqual({ sessionId: expect.any(String), roots: createRoots(["C:\\gamedata"]) });
  });

  it("comes back to the roots the backend is still browsing after a reload", async () => {
    // The session lives where every other application's does; a reload asks for it and derives the listing again, which
    // is cheap because the mounts that listing reads are already cached.
    mockIpcCommands({
      [EVisualsCommand.OPEN_BROWSE]: mockSessionResponse(({ roots }: { roots: XrayRoots }) => roots),
      [EVisualsCommand.RESTORE_BROWSE]: mockSessionResponse(createRoots(["C:\\gamedata"])),
      [EAssetsCommand.LIST_ASSETS]: [mockVisual("meshes\\actors\\stalker.ogf")],
    });

    const { service } = mockInjectedService(VisualsBrowseService);

    await service.onProvision();

    expect(service.rootsLabel).toBe("C:\\gamedata");
    expect(service.visuals.value ?? []).toHaveLength(1);
  });

  it("forgets the browsed roots as its tool is left, and the tool's scope closes them in place", async () => {
    // The selection is dropped on the way out, so a session left open would come back beside a model the backend no
    // longer has. A reload releases no scope, which is what keeps the restore above working.
    const released: Array<string> = [];

    mockIpcCommands({
      [EVisualsCommand.OPEN_BROWSE]: mockSessionResponse(({ roots }: { roots: XrayRoots }) => roots),
      [EAssetsCommand.LIST_ASSETS]: [mockVisual("meshes\\actors\\stalker.ogf")],
      [EVisualsCommand.CLOSE_BROWSE]: () => void released.push("closed"),
      [ESessionsCommand.RELEASE_SCOPE]: () => {
        released.push("scope released");

        return 1;
      },
    });

    const { service, container } = mockInjectedService(VisualsBrowseService);

    await service.openRoot("C:\\gamedata");

    container.get(SessionScopeService).release();

    expect(service.rootsLabel).toBeNull();
    expect(service.isBrowsing).toBe(false);
    expect(released).toEqual(["scope released"]);
  });

  it("forgets the roots when browsing is closed, backend included", async () => {
    const closed: Array<string> = [];

    mockIpcCommands({
      [EVisualsCommand.OPEN_BROWSE]: mockSessionResponse(({ roots }: { roots: XrayRoots }) => roots),
      [EAssetsCommand.LIST_ASSETS]: [],
      [EVisualsCommand.CLOSE_BROWSE]: () => {
        closed.push("closed");
      },
      // A session the backend no longer holds is what a later provisioning must find.
      [EVisualsCommand.RESTORE_BROWSE]: mockSessionResponse(null),
    });

    const { service } = mockInjectedService(VisualsBrowseService);

    await service.openRoot("C:\\gamedata");
    await service.close();

    expect(service.rootsLabel).toBeNull();
    expect(service.isBrowsing).toBe(false);
    expect(service.rootPaths).toEqual([]);
    expect(closed).toEqual(["closed"]);

    const next = mockInjectedService(VisualsBrowseService);

    await next.service.onProvision();

    expect(next.service.rootsLabel).toBeNull();
  });
});
