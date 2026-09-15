import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback } from "react";

import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { TArchiveContent, useLastContent } from "@/core/archive/lib";
import { ArchiveDescribeRefusal, ArchiveFileDescription } from "@/core/ipc/types/xrf-app";
import { DelayedProgress } from "@/core/ui/layout/DelayedProgress";
import { EmptyState } from "@/core/ui/layout/EmptyState";
import { AsyncState } from "@/lib/async-state";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatBytes } from "@/lib/memory/format";
import { Nullable } from "@/lib/types/general";

import { ArchivePreviewError } from "../ArchivePreviewError";
import { ArchiveThmDescriptionView } from "./ArchiveThmDescriptionView";

/**
 * What the backend can say about a binary entry the viewer cannot draw.
 */
export function ArchiveDescriptionPreview({
  "data-testid": dataTestId = "archive-description-preview",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const archivesService: ArchivesService = useInjection(ArchivesService);
  const content: AsyncState<Nullable<TArchiveContent>> = archivesService.content;

  // The previous description stays on screen while the next one is read, rather than the panel blanking between
  // clicks - the same courtesy the texture and sound previews extend.
  const described: Nullable<TArchiveContent & { kind: "description" }> = useLastContent(
    content.value?.kind === "description" ? content.value : null,
    content.isLoading
  );

  const onGetRefusalDescription = useCallback((reason: ArchiveDescribeRefusal): string => {
    switch (reason.kind) {
      case "noDescriber":
        return reason.extension
          ? `Nothing reads .${reason.extension} files yet. Their metadata is still available in Details.`
          : "Nothing reads files without an extension yet. Their metadata is still available in Details.";
      case "tooLarge":
        return (
          `This file is ${formatBytes(reason.size)}, past the ${formatBytes(reason.maximum)} limit for reading a ` +
          "file whole to describe it. Its metadata is still available in Details."
        );
    }
  }, []);

  if (content.error) {
    return (
      <ArchivePreviewError
        data-testid={dataTestId}
        id={id}
        className={className}
        error={content.error}
        onRetry={archivesService.retrySelectedFile}
      />
    );
  }

  if (!described) {
    return content.isLoading ? (
      <DelayedProgress data-testid={dataTestId} id={id} className={className} />
    ) : (
      <EmptyState
        data-testid={dataTestId}
        id={id}
        className={className}
        title={"Nothing to show"}
        description={"The selected file did not return a description."}
      />
    );
  }

  const description: ArchiveFileDescription = described.description;

  switch (description.format.kind) {
    case "thm":
      return (
        <ArchiveThmDescriptionView
          data-testid={dataTestId}
          id={id}
          className={className}
          description={description.format.description}
          scope={description.scope}
        />
      );

    case "unsupported":
      return (
        <EmptyState
          data-testid={dataTestId}
          id={id}
          className={className}
          title={"No description yet"}
          description={onGetRefusalDescription(description.format.reason)}
        />
      );
  }
}
