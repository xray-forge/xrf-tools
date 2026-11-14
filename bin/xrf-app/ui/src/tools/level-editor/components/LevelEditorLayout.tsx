import { Nullable } from "@xrf/types";
import { EditorFileHeader } from "@xrf/ui/editor/file-header";
import { EditorLayout } from "@xrf/ui/editor/layout";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { AccountTreeIcon } from "@xrf/ui/icons/account-tree";
import { InfoOutlinedIcon } from "@xrf/ui/icons/info-outlined";
import { TuneIcon } from "@xrf/ui/icons/tune";
import { WarningAmberIcon as WarningIcon } from "@xrf/ui/icons/warning-amber";
import { cn } from "@xrf/ui/theme";
import { ReactElement } from "react";

import { EditorToolbar } from "@/core/shell/chrome/EditorToolbar";
import { useShellPanels } from "@/core/shell/contributions";
import { IShellPanel } from "@/core/shell/panels/shell-panel";
import { LevelEditorInspectorPanel } from "@/tools/level-editor/components/panels/LevelEditorInspectorPanel";
import { LevelEditorOutlinerPanel } from "@/tools/level-editor/components/panels/LevelEditorOutlinerPanel";
import { LevelEditorProblemsPanel } from "@/tools/level-editor/components/panels/LevelEditorProblemsPanel";
import { LevelEditorScenePanel } from "@/tools/level-editor/components/panels/LevelEditorScenePanel";
import { LevelEditorViewportState } from "@/tools/level-editor/components/viewport/LevelEditorViewportState";
import { ELevelEditorPanelId } from "@/tools/level-editor/lib/panels/level-editor-panel-id";

interface ILevelEditorLayoutProps extends BaseComponentProps {
  /** What the open scene is called, which heads the workspace. */
  name?: Nullable<string>;
  /** What the scene being opened is called, which heads the workspace until it is open. */
  pending?: Nullable<string>;
  /** Whether a scene is being opened. */
  isLoading?: boolean;
  /** Whether a change moved what the open scene was read from, which it is shown as until it is read again. */
  isStale?: boolean;
  /** Why the last open failed, shown with `onRetry` while no scene is open. */
  error?: string;
  onRetry?: () => void;
  onBack?: () => void;
  onClose?: Nullable<() => void>;
}

/**
 * The level editor's chrome: the toolbar, the scene's header, the place the scene will be drawn in, and the panel
 * stripe listing, inspecting and checking what the scene holds.
 */
export function LevelEditorLayout({
  "data-testid": dataTestId = "level-editor-layout",
  id = "level-editor-layout",
  className,
  name = null,
  pending = null,
  isLoading = false,
  isStale = false,
  error,
  onRetry,
  onBack,
  onClose = null,
}: ILevelEditorLayoutProps): ReactElement {
  const heading: Nullable<string> = name ?? pending;

  useShellPanels(
    (): Array<IShellPanel> => [
      {
        icon: <AccountTreeIcon />,
        id: ELevelEditorPanelId.OUTLINER,
        isOpenByDefault: true,
        label: "Outliner",
        render: () => <LevelEditorOutlinerPanel />,
        side: "left",
      },
      {
        icon: <TuneIcon />,
        id: ELevelEditorPanelId.INSPECTOR,
        isOpenByDefault: true,
        label: "Inspector",
        render: () => <LevelEditorInspectorPanel />,
      },
      {
        icon: <InfoOutlinedIcon />,
        id: ELevelEditorPanelId.SCENE,
        label: "Scene",
        render: () => <LevelEditorScenePanel />,
      },
      {
        icon: <WarningIcon />,
        id: ELevelEditorPanelId.PROBLEMS,
        label: "Problems",
        render: () => <LevelEditorProblemsPanel />,
      },
    ],
    []
  );

  return (
    <EditorLayout toolbar={<EditorToolbar onBack={onBack} />}>
      <div className={"flex min-h-0 min-w-0 grow flex-col"}>
        {heading ? (
          <EditorFileHeader
            data-testid={"level-editor-file-header"}
            name={heading}
            caption={isStale ? "Changed since read" : undefined}
            closeLabel={"Close scene"}
            closeDescription={"Close this scene"}
            onClose={onClose ?? undefined}
          />
        ) : null}

        <div
          data-testid={dataTestId}
          id={id}
          className={cn("relative flex min-h-0 min-w-0 flex-1 overflow-hidden", className)}
        >
          <LevelEditorViewportState isLoading={isLoading} error={name ? undefined : error} onRetry={onRetry} />
        </div>
      </div>
    </EditorLayout>
  );
}
