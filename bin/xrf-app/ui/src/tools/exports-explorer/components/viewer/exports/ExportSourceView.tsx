import { useInjection } from "@wirestate/react";
import { AsyncState } from "@xrf/async";
import { transformError } from "@xrf/errors";
import { SessionId } from "@xrf/ipc/types/xrf-app-core";
import { ExportSourceContent } from "@xrf/ipc/types/xrf-export";
import { Nullable } from "@xrf/types";
import { CodeView } from "@xrf/ui/code/view";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ContentState } from "@xrf/ui/layout/content-state";
import { cn } from "@xrf/ui/theme";
import { ReactElement, useCallback, useEffect, useState } from "react";

import { getSyntaxLanguage } from "@/core/syntax/lib/syntax-language";
import { ExportsService } from "@/tools/exports-explorer/services/exports.service";

export interface IExportSourceViewProps extends BaseComponentProps {
  name: string;
}

/**
 * The source that declares one extern, read back from the project on demand.
 */
export function ExportSourceView({
  "data-testid": dataTestId,
  id,
  className,
  name,
}: IExportSourceViewProps): ReactElement {
  const exportsService: ExportsService = useInjection(ExportsService);

  // A refresh opens the project again under a new session, whose source may have moved since.
  const sessionId: Nullable<SessionId> = exportsService.project.sessionId;

  const [source, setSource] = useState<AsyncState<Nullable<ExportSourceContent>>>(() =>
    AsyncState.loading<ExportSourceContent>()
  );
  const [retry, setRetry] = useState(0);

  const onRetry = useCallback(() => setRetry((current) => current + 1), []);

  useEffect(() => {
    let isActive: boolean = true;

    setSource((current) => current.asLoading(null));

    exportsService
      .readExportSource(name)
      .then((result: ExportSourceContent) => isActive && setSource((current) => current.asReady(result)))
      .catch((error: unknown) => isActive && setSource((current) => current.asFailed(transformError(error), null)));

    // Reads need not come back in order, so one abandoned by a newer selection is dropped here.
    return () => {
      isActive = false;
    };
  }, [exportsService, name, retry, sessionId]);

  return (
    <ContentState
      data-testid={dataTestId}
      id={id}
      className={className}
      value={source.value}
      isLoading={source.isLoading}
      error={source.error}
      loadingLabel={"Reading export source…"}
      errorTitle={"Could not read this source"}
      onRetry={onRetry}
      emptyTitle={"Source unavailable"}
      emptyDescription={"The source of this declaration could not be read."}
    >
      {(content: ExportSourceContent) => (
        <CodeView
          data-testid={dataTestId}
          id={id}
          className={cn("rounded-surface border border-divider bg-well", className)}
          label={`Source of ${content.name}`}
          content={content.content}
          language={getSyntaxLanguage(content.path)}
          firstLine={content.line}
        />
      )}
    </ContentState>
  );
}
