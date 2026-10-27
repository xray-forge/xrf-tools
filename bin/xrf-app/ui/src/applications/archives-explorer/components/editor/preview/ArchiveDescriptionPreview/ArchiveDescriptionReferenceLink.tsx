import { useInjection } from "@wirestate/react";
import { ArchiveReference } from "@xrf/ipc/types/xrf-app-archives";
import { Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { Link } from "@xrf/ui/material";
import { cn } from "@xrf/ui/theme/class-name";
import { ReactElement, useCallback } from "react";

import { ArchivePreviewService } from "@/applications/archives-explorer/services/archive-preview";

interface IArchiveDescriptionReferenceLinkProps extends BaseComponentProps {
  reference: ArchiveReference;
}

/**
 * The name a description gives a file, selectable in the tree when the open subject holds it.
 */
export function ArchiveDescriptionReferenceLink({
  "data-testid": dataTestId = "archive-description-reference-link",
  id,
  className,
  reference,
}: IArchiveDescriptionReferenceLinkProps): ReactElement {
  const previewService: ArchivePreviewService = useInjection(ArchivePreviewService);

  const entry: Nullable<string> = reference.entry;

  const onOpenReference = useCallback(() => {
    if (entry) {
      previewService.selectFileByName(entry);
    }
  }, [entry, previewService]);

  if (!entry) {
    return (
      <span data-testid={dataTestId} id={id} className={className}>
        {reference.name}
      </span>
    );
  }

  return (
    <Link
      data-testid={dataTestId}
      id={id}
      className={cn("align-baseline [font:inherit]", className)}
      component={"button"}
      type={"button"}
      underline={"hover"}
      onClick={onOpenReference}
    >
      {reference.name}
    </Link>
  );
}
