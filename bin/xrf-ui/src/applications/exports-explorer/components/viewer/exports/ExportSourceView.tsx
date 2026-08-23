import { Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useEffect, useState } from "react";

import { ExportsService } from "@/applications/exports-explorer/services/exports";
import { ExportSourceContent } from "@/core/bindings/types/xrf-export";
import { transformError } from "@/core/error/lib";
import { CodeView } from "@/core/syntax/components/CodeView";
import { getSyntaxLanguage } from "@/core/syntax/lib";
import { DelayedProgress } from "@/core/ui/layout/DelayedProgress";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { createLoadable, Loadable } from "@/lib/loadable";
import { Nullable } from "@/lib/types/general";

export interface IExportSourceViewProps extends BaseComponentProps {
  name: string;
}

/**
 * The source that declares one extern, read back from the project on demand.
 */
export function ExportSourceView({ name }: IExportSourceViewProps): ReactElement {
  const exportsService: ExportsService = useInjection(ExportsService);

  const [source, setSource] = useState<Loadable<Nullable<ExportSourceContent>>>(() => createLoadable(null, true));

  useEffect(() => {
    let isActive: boolean = true;

    setSource(createLoadable(null, true));

    exportsService
      .readExportSource(name)
      .then((result: ExportSourceContent) => isActive && setSource(createLoadable(result)))
      .catch((error: unknown) => isActive && setSource(createLoadable(null, false, transformError(error))));

    // Reads need not come back in order, so one abandoned by a newer selection is dropped here.
    return () => {
      isActive = false;
    };
  }, [exportsService, name]);

  if (source.isLoading) {
    return <DelayedProgress />;
  } else if (source.error) {
    return (
      <Typography variant={"body2"} sx={{ color: "error.main" }}>
        {String(source.error)}
      </Typography>
    );
  }

  return source.value ? (
    <CodeView
      label={`Source of ${source.value.name}`}
      content={source.value.content}
      language={getSyntaxLanguage(source.value.path)}
      firstLine={source.value.line}
      sx={{ borderRadius: 1, backgroundColor: "background.default" }}
    />
  ) : (
    <Typography variant={"body2"} sx={{ color: "text.secondary" }}>
      The source of this declaration could not be read.
    </Typography>
  );
}
