import { Box, Typography } from "@mui/material";
import { ReactElement, useMemo } from "react";

import { ARCHIVE_PATH_TEXT } from "@/applications/archives-explorer/components/editor/archive-editor.styles";
import { listPayloadSharersOf } from "@/core/archive/files";
import { ArchiveFileDescriptor, ArchiveSharedPayload } from "@/core/bindings/types/xrf-archive";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Loadable } from "@/lib/loadable";

interface IArchiveSharedPayloadDetailProps extends BaseComponentProps {
  descriptor: ArchiveFileDescriptor;
  sharedPayloads: Loadable<Array<ArchiveSharedPayload>>;
}

/**
 * Which other entries read the selected file's bytes.
 */
export function ArchiveSharedPayloadDetail({
  descriptor,
  sharedPayloads,
}: IArchiveSharedPayloadDetailProps): ReactElement {
  const others: Array<string> = listPayloadSharersOf(sharedPayloads.value ?? [], descriptor);

  const description = useMemo(() => {
    if (sharedPayloads.isLoading) {
      return "Deriving from the name table...";
    }

    if (sharedPayloads.error) {
      return "Could not derive which entries read the same bytes.";
    }

    if (!others.length) {
      return "No other entry reads these bytes";
    }

    return `${
      others.length
    } other entr${others.length === 1 ? "y reads" : "ies read"} these bytes, derived from equal descriptors`;
  }, [others.length, sharedPayloads.error, sharedPayloads.isLoading]);

  return (
    <Box sx={{ marginBottom: 1.5 }}>
      <Typography variant={"caption"} sx={{ display: "block", color: "text.secondary" }}>
        Shared payload
      </Typography>

      <Typography variant={"body2"} sx={{ overflowWrap: "anywhere" }}>
        {description}
      </Typography>

      {others.map((name: string) => (
        <Typography key={name} variant={"body2"} sx={ARCHIVE_PATH_TEXT}>
          {name}
        </Typography>
      ))}
    </Box>
  );
}
