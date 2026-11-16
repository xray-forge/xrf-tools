import { TuneIcon } from "@xrf/ui/icons/tune";

import {
  ARCHIVE_CONFIG_SECTIONS_LABEL,
  ArchiveConfigSectionsMenu,
  IArchiveConfigSection,
} from "@/core/archive/components/config-editor/ArchiveConfigSectionsMenu";
import { IArchiveConfigEditor } from "@/core/archive/services/archive-config-editor.service";
import { useShellPanels, useShellStatus } from "@/core/shell/contributions";
import { useToolLifecycle } from "@/core/shell/lifecycle";

export interface IArchiveConfigEditorShellOptions<TSection extends string> {
  editor: IArchiveConfigEditor<TSection>;
  /** The id the shell keeps the sections panel under. */
  panelId: string;
  /** The sections, in the order they are edited; a module constant, since the panel registers once per value. */
  sections: ReadonlyArray<IArchiveConfigSection<TSection>>;
  /** Whether configuration I/O or a run is under way, which holds the tool open. */
  isBusy: boolean;
  /** What the last run produced, after the configuration's name and state in the status bar. */
  resultStatus: ReadonlyArray<string>;
}

/**
 * What an archive configuration editor publishes to the shell: the section navigation as a left panel beside every
 * other tool's navigation, the configuration's name and state in the status bar, and the busy and unsaved state that
 * holds the tool open. Unsaved edits have nowhere to go but a file the user picks, so leaving offers no save.
 */
export function useArchiveConfigEditorShell<TSection extends string>({
  editor,
  panelId,
  sections,
  isBusy,
  resultStatus,
}: IArchiveConfigEditorShellOptions<TSection>): void {
  // The menu reads the open section from the editor, so this registers once.
  useShellPanels(
    () => [
      {
        icon: <TuneIcon />,
        id: panelId,
        isOpenByDefault: true,
        label: ARCHIVE_CONFIG_SECTIONS_LABEL,
        render: () => <ArchiveConfigSectionsMenu editor={editor} sections={sections} />,
        side: "left",
      },
    ],
    [editor, panelId, sections]
  );

  useShellStatus([
    editor.configName ?? "no configuration",
    ...(editor.isDirty ? ["unsaved changes"] : []),
    ...resultStatus,
  ]);

  useToolLifecycle({
    isBusy,
    dirtyCount: editor.isDirty ? 1 : 0,
    save: null,
  });
}
