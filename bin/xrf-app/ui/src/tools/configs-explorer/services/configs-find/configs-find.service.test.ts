import { beforeEach, describe, expect, it } from "@jest/globals";
import { EConfigsCommand } from "@xrf/ipc/commands/configs";
import { mockIpcCommands, resetMockInvoke } from "@xrf/ipc/testing";
import { ConfigsDeclarationSearch, ConfigsProjectDescriptor } from "@xrf/ipc/types/xrf-app-configs";

import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { ConfigsProjectService } from "@/tools/configs-explorer/services/configs-project";

import { ConfigsFindService } from "./configs-find.service";

/** A project open, and a search of it answering one statement; the close answers null, as anything unlisted does. */
function armProject(): void {
  mockIpcCommands({
    [EConfigsCommand.OPEN_PROJECT]: mockSessionResponse({} as ConfigsProjectDescriptor),
    [EConfigsCommand.FIND_DECLARATIONS]: mockSessionResponse({ total: 1 } as ConfigsDeclarationSearch),
  });
}

describe("ConfigsFindService", () => {
  beforeEach(() => {
    resetMockInvoke();
  });

  // Rows open a file at a line: kept past their project, they open another project's file, or a line since moved.
  it("forgets its search once the project it answered goes, however it went", async () => {
    const { container, service } = mockInjectedService(ConfigsFindService, [ConfigsProjectService]);
    const projectService: ConfigsProjectService = container.get(ConfigsProjectService);

    armProject();
    service.onProvision();

    await projectService.open("C:\\game\\gamedata", false);
    service.setQuery({ section: "wpn_ak74" });
    await service.find();

    expect(service.results.value).not.toBeNull();

    await projectService.close();

    expect(service.results.value).toBeNull();
    expect(service.query.section).toBe("");

    service.onDeprovision();
  });
});
