import type { Meta, StoryObj } from "@storybook/react-vite";
import { LOGICAL_PATH_SEPARATOR } from "@xrf/path";
import { ESyntaxLanguage } from "@xrf/syntax";
import { Nullable } from "@xrf/types";
import { ReactElement, useCallback, useState } from "react";
import { fn } from "storybook/test";

import { CodeView } from "#/code/view";
import { EditorFileHeader } from "#/editor/file-header";
import { EditorIconAction } from "#/editor/icon-action";
import { EditorPanelProperty, EditorPanelSection } from "#/editor/panel";
import { EditorToolbarCrumb, EditorToolbarHostContext, EditorToolbarSeparator } from "#/editor/toolbar";
import { ArchiveIcon } from "#/icons/archive";
import { DescriptionOutlinedIcon } from "#/icons/description-outlined";
import { FolderIcon } from "#/icons/folder";
import { FolderOpenIcon } from "#/icons/folder-open";
import { RefreshIcon } from "#/icons/refresh";
import { SearchIcon } from "#/icons/search";
import { SettingsIcon } from "#/icons/settings";
import { TerminalIcon } from "#/icons/terminal";
import { RailButton } from "#/shell/rail-button";
import { PANEL } from "#/theme";
import {
  getFileItemPath,
  IPathTreeItem,
  ITreeNode,
  IVirtualizedTreeIcons,
  parsePathTree,
  toDirectoryItemId,
  toFileItemId,
  useTreeState,
  VirtualizedTree,
} from "#/tree";

import { EditorLayout } from "./EditorLayout";

const SOURCES: Readonly<Record<string, string>> = {
  "config\\system.ltx": `[actor]\n#include "creatures\\\\actor.ltx"\n\n[mp_actor]:actor\ncost = 0 ; never traded\n`,
  "config\\creatures\\actor.ltx": `[actor]:identity_immunities\nhealth_restore_v = 0.0001\nmax_item_mass = 50\n`,
  "config\\weapons\\w_ak74.ltx": `[wpn_ak74]\nammo_mag_size = 30\nrpm = 600\n`,
};

const ITEMS: Array<IPathTreeItem<string>> = parsePathTree(
  Object.keys(SOURCES).map((path: string) => ({ path, payload: path })),
  LOGICAL_PATH_SEPARATOR
);

const EXPANDED_IDS: ReadonlyArray<string> = [toDirectoryItemId("config"), toDirectoryItemId("config\\creatures")];

const INITIAL_OPEN_PATH: string = "config\\creatures\\actor.ltx";

const TREE_ICONS: IVirtualizedTreeIcons = {
  collapsed: <FolderIcon />,
  expanded: <FolderOpenIcon />,
  leaf: <DescriptionOutlinedIcon />,
};

/** What the chrome around the editor does on a click; the pattern shows it, the application acts on it. */
const ON_CHROME_CLICK: () => void = fn();

const RELOAD_ACTION: ReactElement = (
  <EditorIconAction
    label={"Reload"}
    description={"Read the archives again"}
    icon={<RefreshIcon />}
    onClick={ON_CHROME_CLICK}
  />
);

const SETTINGS_ACTION: ReactElement = (
  <EditorIconAction
    label={"Settings"}
    description={"Open the settings"}
    icon={<SettingsIcon />}
    onClick={ON_CHROME_CLICK}
  />
);

const STATUS_BAR: ReactElement = (
  <div className={"flex h-status-bar shrink-0 items-center surface-frame px-3 text-xs text-text-secondary"}>
    3 files in 1 archive
  </div>
);

function EditorShellPattern(): ReactElement {
  const [toolbarHost, setToolbarHost] = useState<Nullable<HTMLElement>>(null);
  const [openPath, setOpenPath] = useState<Nullable<string>>(INITIAL_OPEN_PATH);
  const tree = useTreeState({ initialExpandedIds: EXPANDED_IDS });
  const { select } = tree;

  const onSelect = useCallback((item: ITreeNode<string>) => select(item.id), [select]);
  const onActivate = useCallback(
    (item: ITreeNode<string>) => setOpenPath((current: Nullable<string>) => getFileItemPath(item.id) ?? current),
    []
  );
  const onClose = useCallback(() => setOpenPath(null), []);

  return (
    <div className={"flex h-160 max-w-300 flex-col overflow-hidden border border-divider surface-frame"}>
      <div className={"flex h-title-bar shrink-0 items-center gap-1 px-2"}>
        <EditorToolbarCrumb label={"Archives"} />
        <EditorToolbarSeparator />
        <div ref={setToolbarHost} className={"flex min-w-0 items-center gap-0.5"} />
      </div>

      <div className={"flex min-h-0 grow"}>
        <div className={"flex w-rail shrink-0 flex-col items-center gap-1 pt-1"}>
          <RailButton label={"Archives"} icon={<ArchiveIcon />} isSelected onClick={ON_CHROME_CLICK} />
          <RailButton label={"Search"} icon={<SearchIcon />} onClick={ON_CHROME_CLICK} />
          <RailButton label={"Console"} icon={<TerminalIcon />} onClick={ON_CHROME_CLICK} />
        </div>

        <div className={"flex min-w-0 grow overflow-hidden rounded-surface border border-divider"}>
          <EditorToolbarHostContext.Provider value={toolbarHost}>
            <EditorLayout toolbar={RELOAD_ACTION} footer={STATUS_BAR}>
              <div className={"shrink-0 border-r border-divider surface-frame"} style={{ width: PANEL.defaultWidth }}>
                <VirtualizedTree<string>
                  items={ITEMS}
                  expandedIds={tree.expandedIds}
                  selectedId={tree.selectedId}
                  activeId={openPath ? toFileItemId(openPath) : null}
                  ariaLabel={"Archive entries"}
                  icons={TREE_ICONS}
                  onToggleExpanded={tree.toggleExpanded}
                  onSelect={onSelect}
                  onActivate={onActivate}
                />
              </div>

              <div className={"flex min-w-0 grow flex-col"}>
                {openPath ? (
                  <>
                    <EditorFileHeader name={openPath} caption={"ltx"} onClose={onClose} />
                    <CodeView className={"grow"} content={SOURCES[openPath]} language={ESyntaxLanguage.LTX} />
                  </>
                ) : null}
              </div>

              <div
                className={"shrink-0 overflow-auto border-l border-divider surface-frame"}
                style={{ width: PANEL.defaultWidth }}
              >
                <EditorPanelSection title={"File"}>
                  <EditorPanelProperty label={"Path"} value={openPath ?? "Nothing open"} isMonospace />
                  <EditorPanelProperty label={"Archive"} value={"configs.db"} isMonospace />
                </EditorPanelSection>
                <EditorPanelSection title={"Settings"} actions={SETTINGS_ACTION}>
                  <EditorPanelProperty label={"Encoding"} value={"windows-1251"} />
                </EditorPanelSection>
              </div>
            </EditorLayout>
          </EditorToolbarHostContext.Provider>
        </div>

        <div className={"w-rail shrink-0"} />
      </div>
    </div>
  );
}

const meta = {
  title: "Patterns/EditorShell",
  component: EditorShellPattern,
} satisfies Meta<typeof EditorShellPattern>;

export default meta;

type TStory = StoryObj<typeof meta>;

/**
 * The window from kit parts: caption and rails on the frame, one inset workspace holding an editor layout with a tree,
 * the open file and a properties panel. Double-click a file to open it.
 */
export const ArchiveEditor: TStory = {};
