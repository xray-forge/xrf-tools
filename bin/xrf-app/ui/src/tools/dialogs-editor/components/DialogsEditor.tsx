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

import { describeRoots } from "@/core/assets/lib";
import { EditorToolbar } from "@/core/shell/editor/EditorToolbar";
import { useEditorBusy } from "@/core/shell/editor-lifecycle";
import { useEditorPanels, useEditorStatus } from "@/core/shell/editor-shell";
import { DialogViewService } from "@/tools/dialogs-editor/services/dialog-view";
import { DialogsService } from "@/tools/dialogs-editor/services/dialogs";

import { DialogsEditorWorkspace } from "./DialogsEditorWorkspace";
import { DialogInspectorPanel } from "./editor/DialogInspectorPanel";
import { DialogsEditorActions } from "./editor/DialogsEditorActions";
import { DialogsProblemsPanel } from "./editor/DialogsProblemsPanel";
import { DialogsTreeMenu } from "./editor/DialogsTreeMenu";

export function DialogsEditor(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const dialogsService: DialogsService = useInjection(DialogsService);
  const viewService: DialogViewService = useInjection(DialogViewService);

  const project: Nullable<DialogProjectDescriptor> = dialogsService.project.value;
  const fileCount: number = Object.keys(project?.files ?? {}).length;
  const dialogCount: number = dialogsService.entries.length;

  const findings: ReadonlyArray<DialogFinding> = useMemo(() => project?.findings ?? [], [project]);

  const onClose = useCallback(async () => {
    log.info("Closing dialogs");

    await dialogsService.closeProject();
  }, [dialogsService, log]);

  useEditorBusy(dialogsService.project.isLoading);

  useEditorPanels(
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
        // Open by default: it is the half of the read-only editor the canvas cannot show, and a graph
        // node carries more than a node is wide.
        isOpenByDefault: true,
        label: "Inspector",
        render: () => <DialogInspectorPanel />,
        side: "right",
      },
      {
        icon: <ReportProblemIcon />,
        id: "problems",
        // Opened when there is something to read, matching the translations editor: a reader that
        // reports rather than refuses is only useful if what it reports is in front of you.
        isOpenByDefault: findings.length > 0,
        label: "Problems",
        render: () => <DialogsProblemsPanel findings={findings} />,
        side: "right",
      },
    ],
    [findings]
  );

  useEditorStatus([
    `${formatCount(fileCount)} ${fileCount === 1 ? "file" : "files"}`,
    `${formatCount(dialogCount)} dialogs`,
    // Zero text keys beside real dialogs is the signature of a layout pointed at the wrong place, so
    // it reads as a state rather than being left to infer from every phrase showing its key.
    project?.textKeys ? `${formatCount(project.textKeys)} text keys` : "no text",
    // The language in force is state, so it reads here rather than from the toolbar action that changes
    // it — the division `EditorToolbar` states in its own contract.
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
      <DialogsEditorWorkspace />
    </EditorLayout>
  );
}
