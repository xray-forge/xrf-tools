import { useInjection } from "@wirestate/react";
import { AsyncState } from "@xrf/async";
import { formatBytes } from "@xrf/format";
import { ARCHIVE_READ_POLICY } from "@xrf/ipc/types/xrf-archive";
import { EPathEntryKind } from "@xrf/path";
import { Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { DelayedProgress } from "@xrf/ui/layout/DelayedProgress";
import { EmptyState } from "@xrf/ui/layout/EmptyState";
import { cn } from "@xrf/ui/theme/class-name";
import { ReactElement, useCallback } from "react";

import { ArchivePreviewService } from "@/applications/archives-explorer/services/archive-preview";
import {
  ArchivePreviewSupport,
  getArchivePreviewSupport,
  IArchiveEntry,
  TArchiveContent,
  TArchiveSelection,
} from "@/core/archive/lib";
import { inline } from "@/lib/callbacks/inline";

import { ArchiveAudioPreview } from "./ArchiveAudioPreview";
import { ArchiveCodePreview } from "./ArchiveCodePreview";
import { ArchiveDescriptionPreview } from "./ArchiveDescriptionPreview";
import { ArchiveDirectoryContent } from "./ArchiveDirectoryContent";
import { ArchiveFileHeader } from "./ArchiveFileHeader";
import { ArchiveImagePreview } from "./ArchiveImagePreview";
import { ArchiveModelPreview } from "./ArchiveModelPreview";
import { ArchivePreviewError } from "./ArchivePreviewError";
import { ArchiveTexturePreview } from "./ArchiveTexturePreview";

// Everything that renders its own preview leaves this union; what is left is a reason to explain.
type TUnsupported = Exclude<
  ArchivePreviewSupport,
  | { kind: "supported" }
  | { kind: "texture" }
  | { kind: "image" }
  | { kind: "audio" }
  | { kind: "model" }
  | { kind: "description" }
>;

interface IArchivesFilePreviewProps extends BaseComponentProps {
  /** Opens the unpack dialog, which is what the tree root offers in place of a directory extraction. */
  onUnpack: () => void;
}

export function ArchivesFilePreview({
  "data-testid": dataTestId = "archives-file-preview",
  id = "archives-file-preview",
  className,
  onUnpack,
}: IArchivesFilePreviewProps): ReactElement {
  const previewService: ArchivePreviewService = useInjection(ArchivePreviewService);

  const selection: TArchiveSelection = previewService.selection;
  const content: AsyncState<Nullable<TArchiveContent>> = previewService.content;

  const onGetUnsupportedDescription = useCallback((support: TUnsupported): string => {
    switch (support.kind) {
      case "too-large":
        return (
          `This file exceeds the ${formatBytes(support.maximumSize)} preview limit. ` +
          "Its metadata is still available in Details."
        );
    }
  }, []);

  // A directory selection is a different kind of thing, not a file that happens to be missing.
  if (selection.kind === EPathEntryKind.DIRECTORY) {
    return <ArchiveDirectoryContent path={selection.path} onUnpack={onUnpack} />;
  }

  const entry: Nullable<IArchiveEntry> = selection.kind === EPathEntryKind.FILE ? selection.entry : null;

  if (!entry) {
    return (
      <EmptyState
        title={"Select a file to preview"}
        description={`Supported text files up to ${formatBytes(ARCHIVE_READ_POLICY.maximumTextSize)} can be displayed.`}
      />
    );
  }

  const support: ArchivePreviewSupport = getArchivePreviewSupport(entry);

  return (
    <div data-testid={dataTestId} id={id} className={cn("flex min-h-0 min-w-0 grow flex-col", className)}>
      <ArchiveFileHeader entry={entry} />

      <div className={"flex min-h-0 min-w-0 grow overflow-hidden"}>
        {inline(() => {
          switch (support.kind) {
            case "texture":
              return <ArchiveTexturePreview />;
            case "image":
              return <ArchiveImagePreview />;
            case "audio":
              return <ArchiveAudioPreview />;
            case "model":
              return <ArchiveModelPreview name={entry.name} />;
            case "description":
              return <ArchiveDescriptionPreview />;
          }

          if (support.kind !== "supported") {
            return <EmptyState title={"Preview unavailable"} description={onGetUnsupportedDescription(support)} />;
          } else if (content.isLoading) {
            return <DelayedProgress />;
          } else if (content.error) {
            return <ArchivePreviewError error={content.error} onRetry={previewService.retry} />;
          } else if (content.value?.kind === "text") {
            return <ArchiveCodePreview file={content.value.result} />;
          }

          return (
            <EmptyState title={"Preview unavailable"} description={"The selected file did not return any content."} />
          );
        })}
      </div>
    </div>
  );
}
