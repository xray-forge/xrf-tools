import { describe, expect, it } from "@jest/globals";
import { CommandBus, CommandType, Container, QueryBus } from "@wirestate/core";
import { Nullable } from "@xrf/types";

import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { ArchivesExplorerKeybindsService } from "@/applications/archives-explorer/services/keybinds";
import { KeybindCommandsService } from "@/core/commands";
import { FOCUS_SEARCH_KEYBIND_COMMAND } from "@/core/search/commands";
import { FOCUS_SEARCH_FIELD_MESSAGE } from "@/core/search/lib";
import {
  IPanelSetActiveCommand,
  PANEL_ACTIVE_QUERY,
  PANEL_SET_ACTIVE_COMMAND,
} from "@/core/shell/panel/panel-messages";
import { mockArchivesVolumes } from "@/fixtures/mocks/archive.mocks";
import { mockRestoredSession, mockSessionSnapshot } from "@/fixtures/mocks/session.mocks";
import { mockContainer, mockInjectedService } from "@/fixtures/utils/container";
import { AsyncState } from "@/lib/async-state";

interface IKeybindsHarness {
  container: Container;
  dispatched: Array<CommandType>;
  openedPanels: Array<IPanelSetActiveCommand>;
  service: ArchivesExplorerKeybindsService;
}

function mockKeybinds(activePanelId: Nullable<string>): IKeybindsHarness {
  const { service, container } = mockInjectedService(ArchivesExplorerKeybindsService, [ArchivesService]);

  const dispatched: Array<CommandType> = [];
  const openedPanels: Array<IPanelSetActiveCommand> = [];

  const commandBus: CommandBus = container.get(CommandBus);

  commandBus.register(PANEL_SET_ACTIVE_COMMAND, (message: IPanelSetActiveCommand) => {
    dispatched.push(PANEL_SET_ACTIVE_COMMAND);
    openedPanels.push(message);
  });

  commandBus.register(FOCUS_SEARCH_FIELD_MESSAGE, () => dispatched.push(FOCUS_SEARCH_FIELD_MESSAGE));
  container.get(QueryBus).register(PANEL_ACTIVE_QUERY, () => activePanelId);

  return { container, dispatched, openedPanels, service };
}

function openSubject(container: Container): void {
  const archivesService: ArchivesService = container.get(ArchivesService);

  archivesService["subjectState"] = AsyncState.ready(
    mockRestoredSession(archivesService, mockSessionSnapshot(mockArchivesVolumes([])))
  );
}

describe("ArchivesExplorerKeybindsService", () => {
  it("opens the file panel before asking for the caret, so the field it holds exists to answer", () => {
    const { dispatched, openedPanels, service } = mockKeybinds(null);

    service.focusSearch();

    expect(dispatched).toEqual([PANEL_SET_ACTIVE_COMMAND, FOCUS_SEARCH_FIELD_MESSAGE]);
    expect(openedPanels).toEqual([{ panelId: "archives", side: "left" }]);
  });

  it("leaves an already open panel alone", () => {
    const { dispatched, service } = mockKeybinds("archives");

    service.focusSearch();

    expect(dispatched).toEqual([FOCUS_SEARCH_FIELD_MESSAGE]);
  });

  it("replaces whatever else that side shows, which is where the tree lives", () => {
    const { openedPanels, service } = mockKeybinds("collisions");

    service.focusSearch();

    expect(openedPanels).toEqual([{ panelId: "archives", side: "left" }]);
  });

  it("is unavailable until an archive is open, so the picker screen keeps its keys", () => {
    // Provisioned rather than merely resolved: provisioning is what binds the handler and its guard to the buses.
    const container: Container = mockContainer([ArchivesService, ArchivesExplorerKeybindsService]).provision();
    const commandsService: KeybindCommandsService = container.get(KeybindCommandsService);

    expect(commandsService.isAvailable(FOCUS_SEARCH_KEYBIND_COMMAND)).toBe(false);
    expect(commandsService.isImplemented(FOCUS_SEARCH_KEYBIND_COMMAND)).toBe(true);

    openSubject(container);

    expect(commandsService.isAvailable(FOCUS_SEARCH_KEYBIND_COMMAND)).toBe(true);
  });
});
