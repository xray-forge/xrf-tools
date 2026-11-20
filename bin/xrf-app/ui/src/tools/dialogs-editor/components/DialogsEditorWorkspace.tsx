import { useInjection } from "@wirestate/react";
import { formatCount } from "@xrf/format";
import { DialogFinding, DialogProjectDescriptor } from "@xrf/ipc/types/xrf-dialog";
import { Logger } from "@xrf/logging";
import { useLogger } from "@xrf/logging/react";
import { Nullable } from "@xrf/types";
import { EditorLayout } from "@xrf/ui/editor/layout";
import { EditorToolbarLocation } from "@xrf/ui/editor/toolbar";
import { AccountTreeIcon } from "@xrf/ui/icons/account-tree";
import { InfoOutlinedIcon } from "@xrf/ui/icons/info-outlined";
import { ReportProblemIcon } from "@xrf/ui/icons/report-problem";
import { ReactElement, useCallback, useMemo } from "react";

import { describeRoots } from "@/core/assets/lib/roots";
import { EditorToolbar } from "@/core/shell/chrome/EditorToolbar";
import { useShellPanels, useShellStatus } from "@/core/shell/contributions";
import { useToolBusy } from "@/core/shell/lifecycle";
import { DialogViewService } from "@/tools/dialogs-editor/services/dialog-view";
import { DialogsService } from "@/tools/dialogs-editor/services/dialogs";

import { DialogGraphView } from "./editor/DialogGraphView";
import { DialogInspectorPanel } from "./editor/DialogInspectorPanel";
import { DialogsEditorActions } from "./editor/DialogsEditorActions";
import { DialogsProblemsPanel } from "./editor/DialogsProblemsPanel";
import { DialogsTreeMenu } from "./editor/DialogsTreeMenu";

export function DialogsEditorWorkspace(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const dialogsService: DialogsService = useInjection(DialogsService);
  const viewService: DialogViewService = useInjection(DialogViewService);

  const project: Nullable<DialogProjectDescriptor> = dialogsService.project.value;
  const fileCount: number = Object.keys(project?.files ?? {}).length;
  const dialogCount: number = dialogsService.entries.length;

  const findings: ReadonlyArray<DialogFinding> = useMemo(() => project?.findings ?? [], [project]);

  const onClose = useCallback(async () => {
    log.info("Closing dialogs");

    await dialogsService.close();
  }, [dialogsService, log]);

  useToolBusy(dialogsService.project.isLoading);

  useShellPanels(
    () => [
      {
        icon: <AccountTreeIcon />,
        id: "dialogs",
        isOpenByDefault: true,
        label: "Dialogs",
        render: () => <DialogsTreeMenu />,
        side: "left",
      },
      {
        icon: <InfoOutlinedIcon />,
        id: "inspector",
        // Open by default: it shows what a graph node is too narrow to carry.
        isOpenByDefault: true,
        label: "Inspector",
        render: () => <DialogInspectorPanel />,
        side: "right",
      },
      {
        icon: <ReportProblemIcon />,
        id: "problems",
        // Opened when there is something to read, as in the translations editor.
        isOpenByDefault: findings.length > 0,
        label: "Problems",
        render: () => <DialogsProblemsPanel findings={findings} />,
        side: "right",
      },
    ],
    [findings]
  );

  useShellStatus([
    `${formatCount(fileCount)} ${fileCount === 1 ? "file" : "files"}`,
    `${formatCount(dialogCount)} dialogs`,
    // Zero text keys beside real dialogs usually means a layout pointed at the wrong place.
    project?.textKeys ? `${formatCount(project.textKeys)} text keys` : "no text",
    // The language in force is state, so the status shows it rather than the toolbar action that changes it.
    ...(viewService.resolvedLanguage ? [viewService.resolvedLanguage] : []),
    ...(findings.length ? [`${formatCount(findings.length)} problems`] : []),
  ]);

  return (
    <EditorLayout
      toolbar={
        <EditorToolbar
          subtitle={project ? <EditorToolbarLocation location={{ path: describeRoots(project.roots) }} /> : null}
          actions={<DialogsEditorActions />}
          onBack={onClose}
        />
      }
    >
      <DialogGraphView />
    </EditorLayout>
  );
}
