import { describe, expect, it } from "@jest/globals";

import { LAUNCHER_HELP } from "@/core/launcher/help";
import { ShellToolService } from "@/core/shell/services/shell-tool";
import { mockInjectedService } from "@/fixtures/utils/container";
import { ARCHIVES_EXPLORER_TOOL } from "@/tools/archives-explorer";
import { LEVEL_VIEWER_TOOL } from "@/tools/level-viewer";

import { HelpService } from "./help.service";

function leave(): void {}

describe("HelpService", () => {
  it("describes the open tool, the launcher, or nothing on a route nothing answers", () => {
    const { container, service } = mockInjectedService(HelpService);
    const shellToolService: ShellToolService = container.get(ShellToolService);

    shellToolService.follow(ARCHIVES_EXPLORER_TOOL, false, leave);

    expect(service.help).toBe(ARCHIVES_EXPLORER_TOOL.help);

    shellToolService.follow(null, true, leave);

    expect(service.help).toBe(LAUNCHER_HELP);

    shellToolService.follow(null, false, leave);

    expect(service.hasHelp).toBe(false);
  });

  it("closes as the screen it was opened on goes, and stays closed on coming back", () => {
    const { container, service } = mockInjectedService(HelpService);
    const shellToolService: ShellToolService = container.get(ShellToolService);

    // Provisioned, as the root container is, which is when the service starts following the screen.
    container.provision();
    shellToolService.follow(ARCHIVES_EXPLORER_TOOL, false, leave);
    service.open();

    expect(service.isOpen).toBe(true);

    shellToolService.follow(LEVEL_VIEWER_TOOL, false, leave);

    expect(service.isOpen).toBe(false);

    shellToolService.follow(ARCHIVES_EXPLORER_TOOL, false, leave);

    expect(service.isOpen).toBe(false);

    container.deprovision();
  });
});
