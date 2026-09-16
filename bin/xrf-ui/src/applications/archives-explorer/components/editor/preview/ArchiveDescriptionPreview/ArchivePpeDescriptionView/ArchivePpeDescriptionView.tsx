import { Box } from "@mui/material";
import { ReactElement } from "react";

import { ArchiveDescribeScope, ArchivePpeColor, ArchivePpeDescription } from "@/core/ipc/types/xrf-app";
import { LAYOUT } from "@/core/theme/tokens";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { ArchivePpeColorSection } from "./ArchivePpeColorSection";
import { ArchivePpeEffectSection } from "./ArchivePpeEffectSection";
import { ArchivePpeValuesSection } from "./ArchivePpeValuesSection";

interface IArchivePpeDescriptionViewProps extends BaseComponentProps {
  description: ArchivePpeDescription;
  scope: ArchiveDescribeScope;
}

/**
 * A post-process effect: how long it runs, and what each of its parameters does over that time.
 */
export function ArchivePpeDescriptionView({
  "data-testid": dataTestId = "archive-ppe-description-view",
  id,
  className,
  description,
  scope,
}: IArchivePpeDescriptionViewProps): ReactElement {
  return (
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={{ flexGrow: 1, minWidth: 0, minHeight: 0, overflowY: "auto" }}
    >
      <Box sx={{ maxWidth: LAYOUT.readingColumnWidth }}>
        <ArchivePpeEffectSection description={description} scope={scope} />

        {description.colors.map((color: ArchivePpeColor) => (
          <ArchivePpeColorSection key={color.name} color={color} />
        ))}

        <ArchivePpeValuesSection values={description.values} colorMap={description.colorMap} />
      </Box>
    </Box>
  );
}
