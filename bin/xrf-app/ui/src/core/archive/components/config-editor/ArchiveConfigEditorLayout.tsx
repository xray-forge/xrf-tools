import { Nullable } from "@xrf/types";
import { EditorLayout } from "@xrf/ui/editor/layout";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { Alert } from "@xrf/ui/feedback";
import { Divider, Stack } from "@xrf/ui/layout/stack";
import { Typography } from "@xrf/ui/typography";
import { ReactElement, ReactNode } from "react";

import { IArchiveConfigEditor } from "@/core/archive/services/archive-config-editor.service";
import { JobProgressView } from "@/core/jobs/components/JobProgressView";
import { IJobState } from "@/core/jobs/lib";
import { EditorToolbar } from "@/core/shell/chrome/EditorToolbar";

interface IArchiveConfigEditorLayoutProps extends BaseComponentProps {
  editor: IArchiveConfigEditor;
  /** The toolbar's actions, named by the edited configuration. */
  actions: ReactNode;
  /** The open section's fields. */
  children: ReactNode;
  /** What the last run produced, or null before one finished. */
  result: ReactNode;
  /** The confirmation a run asks first. */
  confirmation: ReactNode;
}

/**
 * The page of an archive configuration editor: the failure, the running job, the open section in a reading column
 * and the last run under it, below a toolbar named for the configuration file.
 */
export function ArchiveConfigEditorLayout({
  "data-testid": dataTestId = "archive-config-editor-layout",
  id,
  className,
  editor,
  actions,
  children,
  result,
  confirmation,
}: IArchiveConfigEditorLayoutProps): ReactElement {
  const job: Nullable<IJobState> = editor.operation.job;

  return (
    <EditorLayout
      data-testid={dataTestId}
      id={id}
      className={className}
      toolbar={<EditorToolbar subtitle={editor.configName ?? "New configuration"} actions={actions} />}
    >
      <div className={"min-w-0 grow overflow-y-auto p-6"}>
        <Stack className={"max-w-reading"} spacing={2}>
          {editor.error ? <Alert severity={"error"}>{editor.error}</Alert> : null}

          {job ? <JobProgressView job={job} onCancel={editor.operation.cancel} /> : null}

          {children}

          {result ? (
            <>
              <Divider />
              <Typography variant={"subtitle2"}>Last run</Typography>
              {result}
            </>
          ) : null}
        </Stack>
      </div>

      {confirmation}
    </EditorLayout>
  );
}
