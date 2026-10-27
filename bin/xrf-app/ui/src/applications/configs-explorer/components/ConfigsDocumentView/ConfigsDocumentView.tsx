import { useInjection } from "@wirestate/react";
import { ConfigsDocument } from "@xrf/ipc/types/xrf-app-configs";
import { Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { cn } from "@xrf/ui/theme/class-name";
import { ReactElement } from "react";

import { ConfigsAuthoredView } from "@/applications/configs-explorer/components/ConfigsDocumentView/ConfigsAuthoredView";
import { ConfigsResolvedView } from "@/applications/configs-explorer/components/ConfigsDocumentView/ConfigsResolvedView";
import {
  ConfigsDocumentService,
  EConfigsDocumentMode,
} from "@/applications/configs-explorer/services/configs-document";
import { EditorFileHeader } from "@/core/shell/editor/EditorFileHeader";
import { DelayedProgress } from "@/core/ui/layout/DelayedProgress";
import { EmptyState } from "@/core/ui/layout/EmptyState";
import { ErrorState } from "@/core/ui/layout/ErrorState";
import { inline } from "@/lib/callbacks/inline";

interface IConfigsDocumentViewProps extends BaseComponentProps {
  /** Ends the selection without closing the project, which the header offers. */
  onDeselect: () => void;
}

/**
 * The config on screen, in whichever of the two views is open.
 */
export function ConfigsDocumentView({
  "data-testid": dataTestId = "configs-document-view",
  id,
  className,
  onDeselect,
}: IConfigsDocumentViewProps): ReactElement {
  const documentService: ConfigsDocumentService = useInjection(ConfigsDocumentService);

  const selected: Nullable<string> = documentService.selected;
  const document: Nullable<ConfigsDocument> = documentService.document.value;

  if (!selected) {
    return (
      <EmptyState
        data-testid={dataTestId}
        id={id}
        className={className}
        title={"No config open"}
        description={"Pick one in the tree to read what it says."}
      />
    );
  }

  return (
    <div data-testid={dataTestId} id={id} className={cn("flex min-h-0 min-w-0 grow flex-col", className)}>
      <EditorFileHeader
        data-testid={"configs-document-header"}
        name={selected}
        closeLabel={"Close config"}
        closeDescription={"Clear the selection and close this config"}
        onClose={onDeselect}
      />

      <div className={"flex min-h-0 min-w-0 grow overflow-hidden"}>
        {inline(() => {
          if (documentService.document.isFailed) {
            return (
              <ErrorState
                title={"Cannot read this config"}
                description={documentService.document.error?.message ?? "The read failed."}
                onRetry={() => void documentService.retry()}
              />
            );
          }

          if (!document) {
            return <DelayedProgress />;
          }

          return documentService.mode === EConfigsDocumentMode.RESOLVED ? (
            <ConfigsResolvedView />
          ) : (
            <ConfigsAuthoredView document={document} />
          );
        })}
      </div>
    </div>
  );
}
