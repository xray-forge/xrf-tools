import { CommandBus, inject, Injectable, QueryBus } from "@wirestate/core";

import { EArchivePanelId } from "@/applications/archives-explorer/components/editor/archive-panels";
import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { KeybindCommand } from "@/core/commands";
import { FOCUS_SEARCH_KEYBIND_COMMAND } from "@/core/search/commands";
import { FOCUS_SEARCH_FIELD_MESSAGE } from "@/core/search/lib";
import {
  IPanelSetActiveMessage,
  IPanelSideMessage,
  PANEL_ACTIVE_QUERY,
  PANEL_SET_ACTIVE_MESSAGE,
} from "@/core/shell/panel/panel-messages";
import { Nullable } from "@/lib/types/general";

/**
 * Answers the archives explorer's commands, shadowing whatever a wider scope binds to the same ones.
 */
@Injectable()
export class ArchivesExplorerKeybindsService {
  public constructor(
    private readonly archivesService: ArchivesService = inject(ArchivesService),
    private readonly commandBus: CommandBus = inject(CommandBus),
    private readonly queryBus: QueryBus = inject(QueryBus)
  ) {}

  /**
   * Hands the caret to the file filter, opening the panel that holds it first.
   */
  @KeybindCommand(FOCUS_SEARCH_KEYBIND_COMMAND, {
    isEnabled: (service: ArchivesExplorerKeybindsService) => service.archivesService.subject.value !== null,
  })
  public focusSearch(): void {
    const active: Nullable<string> =
      this.queryBus.query<Nullable<string>, IPanelSideMessage>(
        PANEL_ACTIVE_QUERY,
        { side: "left" },
        { optional: true }
      ) ?? null;

    // The panel command renders synchronously, so the field it holds is mounted and listening by the next line.
    if (active !== EArchivePanelId.FILES) {
      this.commandBus.execute<void, IPanelSetActiveMessage>(
        PANEL_SET_ACTIVE_MESSAGE,
        { panelId: EArchivePanelId.FILES, side: "left" },
        { optional: true }
      );
    }

    this.commandBus.execute(FOCUS_SEARCH_FIELD_MESSAGE, undefined, { optional: true });
  }
}
