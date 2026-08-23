import { default as ReportProblemIcon } from "@mui/icons-material/ReportProblem";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useMemo } from "react";

import { TranslationsEditorActions } from "@/applications/translations-editor/components/editor/TranslationsEditorActions";
import { TranslationsProblemsPanel } from "@/applications/translations-editor/components/editor/TranslationsProblemsPanel";
import { TranslationsEditorWorkspace } from "@/applications/translations-editor/components/TranslationsEditorWorkspace";
import { TranslationsService } from "@/applications/translations-editor/services/translations";
import { TranslationFinding, TranslationProjectDescriptor } from "@/core/bindings/types/xrf-translation";
import { EditorLayout } from "@/core/shell/editor/EditorLayout";
import { EditorToolbar } from "@/core/shell/editor/EditorToolbar";
import { useEditorDirty } from "@/core/shell/EditorDirtyContext";
import { useEditorStatus } from "@/core/shell/EditorStatusContext";
import { useEditorPanels } from "@/core/shell/panel/context";
import { Logger, useLogger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

export function TranslationsEditor(): ReactElement {
  const log: Logger = useLogger("translations");

  const translationsService: TranslationsService = useInjection(TranslationsService);

  const project: Nullable<TranslationProjectDescriptor> = translationsService.project.value;
  const fileCount: number = Object.keys(project?.files ?? {}).length;
  const dirtyCount: number = translationsService.dirtyFiles.length;

  const findings: ReadonlyArray<TranslationFinding> = useMemo(() => project?.findings ?? [], [project]);

  const onClose = useCallback(async () => {
    log.info("Closing translations");

    await translationsService.closeProject();
  }, [log, translationsService]);

  useEditorPanels(
    () => [
      {
        icon: <ReportProblemIcon />,
        id: "problems",
        // Opened when there is something to read: a reader that reports rather than refuses is only
        // useful if what it reports is in front of you.
        isOpenByDefault: findings.length > 0,
        label: "Problems",
        render: () => <TranslationsProblemsPanel findings={findings} />,
        side: "right",
      },
    ],
    [findings]
  );

  useEditorStatus([
    `${fileCount} files`,
    `${project?.languages.length ?? 0} languages`,
    ...(dirtyCount ? [`${dirtyCount} unsaved`] : []),
    ...(project?.findings.length ? [`${project.findings.length} problems`] : []),
  ]);

  useEditorDirty(dirtyCount);

  return (
    <EditorLayout toolbar={<EditorToolbar actions={<TranslationsEditorActions />} onBack={onClose} />}>
      <TranslationsEditorWorkspace />
    </EditorLayout>
  );
}
