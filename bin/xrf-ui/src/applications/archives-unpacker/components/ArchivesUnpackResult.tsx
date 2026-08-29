import { GridColDef } from "@mui/x-data-grid";
import { ReactElement, useMemo } from "react";

import { ArchiveUnpackResult } from "@/core/bindings/types/xrf-pack";
import { EApplicationId } from "@/core/routing/application";
import { CommandResult, ICommandResultStat } from "@/core/ui/command-result/CommandResult";
import { CommandResultFindings } from "@/core/ui/command-result/CommandResultFindings";
import { RevealPathButton } from "@/core/ui/reveal/RevealPathButton";
import { formatDuration } from "@/lib/format/duration";
import { formatBytes } from "@/lib/memory/format";
import { Nullable } from "@/lib/types/general";

interface IArchivesUnpackResultProps {
  result: ArchiveUnpackResult;
  /** Where the run was told to write. The result's rendered `destination` is display text, not an address. */
  outputPath: Nullable<string>;
}

export function ArchivesUnpackResult({ result, outputPath }: IArchivesUnpackResultProps): ReactElement {
  const columns: Array<GridColDef> = useMemo(
    () => [{ field: "archive", headerName: "Archive", flex: 1, minWidth: 320, cellClassName: "monospace" }],
    []
  );

  const rows: Array<{ archive: string }> = useMemo(
    () => result.archives.map((archive) => ({ archive })),
    [result.archives]
  );

  const stats: Array<ICommandResultStat> = useMemo(
    () => [
      { label: "archives", value: result.archives.length },
      { label: "unpacked", value: formatBytes(result.unpackedSize) },
      { label: "prepare", value: formatDuration(result.prepareDuration) },
      { label: "unpack", value: formatDuration(result.unpackDuration) },
      { label: "elapsed", value: formatDuration(result.duration) },
    ],
    [result]
  );

  return (
    <CommandResult
      headline={`Unpacked ${result.archives.length} archive(s) to ${result.destination}`}
      tone={"success"}
      stats={stats}
      actions={
        <RevealPathButton application={EApplicationId.ARCHIVES_UNPACKER} path={outputPath} label={"Show output"} />
      }
    >
      <CommandResultFindings<{ archive: string }>
        rows={rows}
        columns={columns}
        getRowId={(row) => row.archive}
        getSearchText={(row) => row.archive}
        emptyLabel={"No archives were unpacked."}
        searchPlaceholder={"Filter by archive"}
      />
    </CommandResult>
  );
}
