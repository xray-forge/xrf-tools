import { Box } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback } from "react";

import { ArchiveAudioPreview } from "@/applications/archives-explorer/components/editor/preview/ArchiveAudioPreview";
import { ArchiveCodePreview } from "@/applications/archives-explorer/components/editor/preview/ArchiveCodePreview";
import { ArchiveDirectoryContent } from "@/applications/archives-explorer/components/editor/preview/ArchiveDirectoryContent";
import { ArchiveFileHeader } from "@/applications/archives-explorer/components/editor/preview/ArchiveFileHeader";
import { ArchiveImagePreview } from "@/applications/archives-explorer/components/editor/preview/ArchiveImagePreview";
import { ArchiveModelPreview } from "@/applications/archives-explorer/components/editor/preview/ArchiveModelPreview";
import { ArchivePreviewError } from "@/applications/archives-explorer/components/editor/preview/ArchivePreviewError";
import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { ArchivePreviewSupport, getArchivePreviewSupport, TArchiveContent, TArchiveSelection } from "@/core/archive";
import { ArchiveFileDescriptor, ArchiveProject } from "@/core/bindings/types/xrf-archive";
import { EPathEntryKind } from "@/core/path/entry-kind";
import { DelayedProgress } from "@/core/ui/layout/DelayedProgress";
import { EmptyState } from "@/core/ui/layout/EmptyState";
import { AsyncState } from "@/lib/async-state";
import { inline } from "@/lib/callbacks/inline";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatBytes } from "@/lib/memory/format";
import { Nullable } from "@/lib/types/general";

// Everything that renders its own preview leaves this union; what is left is a reason to explain.
type TUnsupported = Exclude<
  ArchivePreviewSupport,
  { kind: "supported" } | { kind: "image" } | { kind: "audio" } | { kind: "model" }
>;

export function ArchivesFileContent({
  "data-testid": dataTestId = "archives-file-content",
  id = "archives-file-content",
  className,
}: BaseComponentProps): ReactElement {
  const archivesService: ArchivesService = useInjection(ArchivesService);

  const selection: TArchiveSelection = archivesService.selection;
  const project: Nullable<ArchiveProject> = archivesService.project.value;
  const content: AsyncState<Nullable<TArchiveContent>> = archivesService.content;

  const onGetUnsupportedDescription = useCallback((support: TUnsupported): string => {
    switch (support.kind) {
      case "unsupported-extension":
        return support.extension
          ? `.${support.extension} files can be inspected in Details, ` +
              "but this file type does not have a text preview."
          : "Files without an extension can be inspected in Details, but do not have a text preview.";
      case "too-large":
        return (
          `This file exceeds the ${formatBytes(support.maximumSize)} preview limit. ` +
          "Its archive metadata is still available in Details."
        );
    }
  }, []);

  // A directory selection is a different kind of thing, not a file that happens to be missing.
  if (selection.kind === EPathEntryKind.DIRECTORY) {
    return <ArchiveDirectoryContent path={selection.path} />;
  }

  const descriptor: Nullable<ArchiveFileDescriptor> =
    selection.kind === EPathEntryKind.FILE ? selection.descriptor : null;

  if (!descriptor || !project) {
    return (
      <EmptyState
        title={"Select a file to preview"}
        description={
          project
            ? `Supported text files up to ${formatBytes(project.readPolicy.maximumSize)} can be displayed.`
            : "Supported text files can be displayed."
        }
      />
    );
  }

  const support: ArchivePreviewSupport = getArchivePreviewSupport(descriptor, project.readPolicy);

  const view = inline(() => {
    switch (support.kind) {
      case "image":
        return <ArchiveImagePreview />;
      case "audio":
        return <ArchiveAudioPreview />;
      case "model":
        return <ArchiveModelPreview key={descriptor.name} name={descriptor.name} />;
    }

    if (support.kind !== "supported") {
      return <EmptyState title={"Preview unavailable"} description={onGetUnsupportedDescription(support)} />;
    } else if (content.isLoading) {
      return <DelayedProgress />;
    } else if (content.error) {
      return <ArchivePreviewError error={content.error} onRetry={archivesService.retrySelectedFile} />;
    } else if (content.value?.kind === "text") {
      return <ArchiveCodePreview file={content.value.result} />;
    }

    return <EmptyState title={"Preview unavailable"} description={"The selected file did not return any content."} />;
  });

  return (
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={{ display: "flex", flexDirection: "column", flexGrow: 1, minWidth: 0, minHeight: 0 }}
    >
      <ArchiveFileHeader descriptor={descriptor} />

      <Box
        sx={{
          display: "flex",
          flexGrow: 1,
          minWidth: 0,
          minHeight: 0,
          overflow: "hidden",
        }}
      >
        {view}
      </Box>
    </Box>
  );
}
