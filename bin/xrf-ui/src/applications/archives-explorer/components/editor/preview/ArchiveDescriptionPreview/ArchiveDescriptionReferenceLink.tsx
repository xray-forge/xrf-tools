import { Link } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { ReactElement, useCallback } from "react";

import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { ArchiveReference } from "@/core/ipc/types/xrf-app";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

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
  const archivesService: ArchivesService = useInjection(ArchivesService);

  const entry: Nullable<string> = reference.entry;

  const onOpenReference = useCallback(() => {
    if (entry) {
      archivesService.openArchiveFileByName(entry);
    }
  }, [archivesService, entry]);

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
