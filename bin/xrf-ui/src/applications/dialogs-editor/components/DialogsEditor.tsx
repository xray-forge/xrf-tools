import { default as ReportProblemIcon } from "@mui/icons-material/ReportProblem";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useMemo } from "react";

import { DialogsEditorWorkspace } from "@/applications/dialogs-editor/components/DialogsEditorWorkspace";
import { DialogsProblemsPanel } from "@/applications/dialogs-editor/components/editor/DialogsProblemsPanel";
import { DialogsService } from "@/applications/dialogs-editor/services/dialogs";
import { DialogFinding, DialogProjectDescriptor } from "@/core/bindings/types/xrf-dialog";
import { EditorLayout } from "@/core/shell/editor/EditorLayout";
import { EditorToolbar } from "@/core/shell/editor/EditorToolbar";
import { useEditorStatus } from "@/core/shell/EditorStatusContext";
import { useEditorPanels } from "@/core/shell/panel/context";
import { Logger, useLogger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

export function DialogsEditor(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const dialogsService: DialogsService = useInjection(DialogsService);

  const project: Nullable<DialogProjectDescriptor> = dialogsService.project.value;
  const fileCount: number = Object.keys(project?.files ?? {}).length;
  const dialogCount: number = useMemo(
    () => Object.values(project?.files ?? {}).reduce((total: number, file) => total + file.dialogs.length, 0),
    [project]
  );

  const findings: ReadonlyArray<DialogFinding> = useMemo(() => project?.findings ?? [], [project]);

  const onClose = useCallback(async () => {
    log.info("Closing dialogs");

    await dialogsService.closeProject();
  }, [dialogsService, log]);

  useEditorPanels(
    () => [
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
    `${fileCount} ${fileCount === 1 ? "file" : "files"}`,
    `${dialogCount} dialogs`,
    // Zero text keys beside real dialogs is the signature of a layout pointed at the wrong place, so
    // it reads as a state rather than being left to infer from every phrase showing its key.
    project?.textKeys ? `${project.textKeys} text keys` : "no text",
    ...(findings.length ? [`${findings.length} problems`] : []),
  ]);

  return (
    <EditorLayout toolbar={<EditorToolbar onBack={onClose} />}>
      <DialogsEditorWorkspace />
    </EditorLayout>
  );
}
