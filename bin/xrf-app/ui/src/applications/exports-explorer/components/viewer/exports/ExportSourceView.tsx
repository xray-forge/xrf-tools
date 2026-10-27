import { useInjection } from "@wirestate/react";
import { AsyncState } from "@xrf/async";
import { transformError } from "@xrf/errors";
import { SessionId } from "@xrf/ipc/types/xrf-app-core";
import { ExportSourceContent } from "@xrf/ipc/types/xrf-export";
import { Nullable } from "@xrf/types";
import { CodeView } from "@xrf/ui/code/view";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { DelayedProgress } from "@xrf/ui/layout/delayed-progress";
import { EmptyState } from "@xrf/ui/layout/empty-state";
import { ErrorState } from "@xrf/ui/layout/error-state";
import { cn } from "@xrf/ui/theme";
import { ReactElement, useCallback, useEffect, useState } from "react";

import { ExportsService } from "@/applications/exports-explorer/services/exports.service";
import { getSyntaxLanguage } from "@/core/syntax/lib/syntax-language";

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

  if (source.isLoading) {
    return <DelayedProgress data-testid={dataTestId} id={id} className={className} label={"Reading export source…"} />;
  } else if (source.error) {
    return (
      <ErrorState
        data-testid={dataTestId}
        id={id}
        className={className}
        title={"Could not read this source"}
        description={source.error.message}
        onRetry={onRetry}
      />
    );
  }

  return source.value ? (
    <CodeView
      data-testid={dataTestId}
      id={id}
      className={cn("rounded-surface border border-divider bg-well", className)}
      label={`Source of ${source.value.name}`}
      content={source.value.content}
      language={getSyntaxLanguage(source.value.path)}
      firstLine={source.value.line}
    />
  ) : (
    <EmptyState
      data-testid={dataTestId}
      id={id}
      className={className}
      title={"Source unavailable"}
      description={"The source of this declaration could not be read."}
    />
  );
}
