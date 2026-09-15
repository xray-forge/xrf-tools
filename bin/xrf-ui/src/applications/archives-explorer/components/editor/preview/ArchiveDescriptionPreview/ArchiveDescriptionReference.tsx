import { Link } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback } from "react";

import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { ArchiveDescribeScope, ArchiveReference } from "@/core/ipc/types/xrf-app";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

import { describeReferenceStatus } from "./ArchiveDescriptionPreview.utils";
import { ArchiveDescriptionRow } from "./ArchiveDescriptionRow";

interface IArchiveDescriptionReferenceProps extends BaseComponentProps {
  label: string;
  reference: ArchiveReference;
  scope: ArchiveDescribeScope;
}

/**
 * A file a description names, selectable in the tree when the open subject holds it.
 */
export function ArchiveDescriptionReference({
  "data-testid": dataTestId = "archive-description-reference",
  id,
  className,
  label,
  reference,
  scope,
}: IArchiveDescriptionReferenceProps): ReactElement {
  const archivesService: ArchivesService = useInjection(ArchivesService);

  const entry: Nullable<string> = reference.entry;

  const onOpenReference = useCallback(() => {
    if (entry) {
      archivesService.openArchiveFileByName(entry);
    }
  }, [archivesService, entry]);

  return (
    <ArchiveDescriptionRow
      data-testid={dataTestId}
      id={id}
      className={className}
      label={label}
      isMonospace
      value={
        entry ? (
          <Link
            component={"button"}
            type={"button"}
            underline={"hover"}
            onClick={onOpenReference}
            sx={{ font: "inherit" }}
          >
            {reference.name}
          </Link>
        ) : (
          reference.name
        )
      }
      caption={describeReferenceStatus(reference, scope)}
    />
  );
}
