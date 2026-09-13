import { Box, Typography } from "@mui/material";
import { ReactElement, useMemo } from "react";

import { ArchiveCompression, ArchiveExtensionUsage } from "@/core/ipc/types/xrf-archive-stats";
import { DetailSection } from "@/core/ui/layout/DetailSection";
import { StatFigure } from "@/core/ui/stats/StatFigure";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatBytes } from "@/lib/memory/format";
import { Nullable } from "@/lib/types/general";

/** One extension and what its payloads compressed to. */
interface IExtensionRatio {
  extension: string;
  ratio: number;
  sizeReal: number;
}

export interface IArchiveCompressionSectionProps extends BaseComponentProps {
  compression: ArchiveCompression;
  /** Read for the per-extension ratios, which only a subject recording stored sizes can offer. */
  extensions: Array<ArchiveExtensionUsage>;
}

/**
 * What the packer achieved, as the format recorded it.
 */
export function ArchiveCompressionSection({
  "data-testid": dataTestId = "archive-compression-section",
  id,
  className,
  compression,
  extensions,
}: IArchiveCompressionSectionProps): ReactElement {
  const ratios: Array<IExtensionRatio> = useMemo(() => {
    return extensions
      .reduce((ratios: Array<IExtensionRatio>, usage: ArchiveExtensionUsage) => {
        const stored: Nullable<number> = usage.sizeCompressed;

        // A group mixing sources reports no stored size, and an empty group has nothing to divide by.
        if (stored !== null && usage.measure.sizeReal > 0) {
          ratios.push({
            extension: usage.extension ?? "(no extension)",
            ratio: stored / usage.measure.sizeReal,
            sizeReal: usage.measure.sizeReal,
          });
        }

        return ratios;
      }, [])
      .sort((first: IExtensionRatio, second: IExtensionRatio) => first.ratio - second.ratio);
  }, [extensions]);

  return (
    <DetailSection
      data-testid={dataTestId}
      id={id}
      className={className}
      title={"Compression"}
      description={
        "What the packer achieved, as the format recorded it. An entry stored at its own size was not compressed at all."
      }
      fact={`ratio ${formatRatio(compression.sizeCompressed, compression.sizeReal)}`}
    >
      <Box sx={{ display: "flex", flexWrap: "wrap", gap: 2, marginBottom: 2 }}>
        <StatFigure label={"Stored"} value={formatBytes(compression.sizeCompressed)} />
        <StatFigure label={"Unpacked"} value={formatBytes(compression.sizeReal)} />
        <StatFigure
          label={"Stored uncompressed"}
          value={compression.storedUncompressed.toLocaleString()}
          hint={"entries"}
        />
      </Box>

      <Typography variant={"caption"} sx={{ color: "text.secondary", display: "block", marginBottom: 1 }}>
        By extension, lowest ratio first - what actually compresses.
      </Typography>

      <Box sx={{ display: "grid", rowGap: 0.5 }}>
        {ratios.map((row: IExtensionRatio) => (
          <Box key={row.extension} sx={{ display: "flex", justifyContent: "space-between", gap: 2, paddingY: 0.25 }}>
            <Typography variant={"caption"}>{row.extension}</Typography>

            <Typography variant={"caption"} sx={{ color: "text.secondary" }}>
              {`${row.ratio.toFixed(3)} · ${formatBytes(row.sizeReal)}`}
            </Typography>
          </Box>
        ))}
      </Box>
    </DetailSection>
  );
}

/**
 * What a payload set compressed to, as stored bytes over unpacked ones.
 *
 * A dash rather than a figure when there is nothing to divide by, because a ratio of zero would read as perfect
 * compression of an empty set.
 */
function formatRatio(sizeCompressed: number, sizeReal: number): string {
  return sizeReal > 0 ? (sizeCompressed / sizeReal).toFixed(3) : "-";
}
