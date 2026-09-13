import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useEffect, useState } from "react";

import { ExportsService } from "@/applications/exports-explorer/services/exports";
import { transformError } from "@/core/error/lib";
import { ExportSourceContent } from "@/core/ipc/types/xrf-export";
import { CodeView } from "@/core/syntax/components/CodeView";
import { getSyntaxLanguage } from "@/core/syntax/lib";
import { DelayedProgress } from "@/core/ui/layout/DelayedProgress";
import { EmptyState } from "@/core/ui/layout/EmptyState";
import { ErrorState } from "@/core/ui/layout/ErrorState";
import { AsyncState } from "@/lib/async-state";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

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
  }, [exportsService, name, retry]);

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
      className={className}
      label={`Source of ${source.value.name}`}
      content={source.value.content}
      language={getSyntaxLanguage(source.value.path)}
      firstLine={source.value.line}
      sx={{ borderRadius: 1, backgroundColor: "background.default" }}
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
