import { flowResult } from "@wirestate/mobx";
import { useInjection } from "@wirestate/react";
import { formatCount } from "@xrf/format";
import { TranslationFinding, TranslationProjectDescriptor } from "@xrf/ipc/types/xrf-translation";
import { Logger } from "@xrf/logging";
import { useLogger } from "@xrf/logging/react";
import { Nullable } from "@xrf/types";
import { EditorLayout } from "@xrf/ui/editor/layout";
import { EditorToolbarLocation } from "@xrf/ui/editor/toolbar";
import { ReportProblemIcon } from "@xrf/ui/icons/report-problem";
import { ReactElement, useCallback, useMemo } from "react";

import { describeRoots } from "@/core/assets/lib";
import { useShellPanels, useShellStatus } from "@/core/shell/contributions";
import { EditorToolbar } from "@/core/shell/editor/EditorToolbar";
import { EditorSaver, useEditorLifecycle } from "@/core/shell/editor-lifecycle";
import { TranslationsService } from "@/tools/translations-editor/services/translations.service";

import { TranslationsEditorActions } from "./editor/TranslationsEditorActions";
import { TranslationsProblemsPanel } from "./editor/TranslationsProblemsPanel";
import { TranslationsEditorWorkspace } from "./TranslationsEditorWorkspace";

export function TranslationsEditor(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const translationsService: TranslationsService = useInjection(TranslationsService);

  const project: Nullable<TranslationProjectDescriptor> = translationsService.project.value;
  const fileCount: number = Object.keys(project?.files ?? {}).length;
  const dirtyCount: number = translationsService.dirtyFiles.length;

  const findings: ReadonlyArray<TranslationFinding> = useMemo(() => project?.findings ?? [], [project]);

  const onClose = useCallback(async () => {
    log.info("Closing translations");

    await translationsService.closeProject();
  }, [log, translationsService]);

  const onSave: EditorSaver = useCallback(() => flowResult(translationsService.saveAll()), [translationsService]);

  useShellPanels(
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

  useShellStatus([
    `${formatCount(fileCount)} ${fileCount === 1 ? "file" : "files"}`,
    `${formatCount(project?.languages.length ?? 0)} languages`,
    ...(dirtyCount ? [`${formatCount(dirtyCount)} unsaved`] : []),
    ...(findings.length ? [`${formatCount(findings.length)} problems`] : []),
  ]);

  useEditorLifecycle({
    isBusy: translationsService.project.isLoading || translationsService.savingFile !== null,
    dirtyCount,
    save: project?.isEditable ? onSave : null,
  });

  return (
    <EditorLayout
      toolbar={
        <EditorToolbar
          subtitle={project ? <EditorToolbarLocation location={{ path: describeRoots(project.roots) }} /> : null}
          actions={<TranslationsEditorActions />}
          onBack={onClose}
        />
      }
    >
      <TranslationsEditorWorkspace />
    </EditorLayout>
  );
}
