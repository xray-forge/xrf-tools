import { GridColDef } from "@mui/x-data-grid";
import { ReactElement, useMemo } from "react";

import { ArchivePackResult } from "@/core/ipc/types/xrf-pack";
import { EApplicationId } from "@/core/routing/application";
import { CommandResult, ICommandResultStat } from "@/core/ui/command-result/CommandResult";
import { CommandResultFindings } from "@/core/ui/command-result/CommandResultFindings";
import { RevealPathButton } from "@/core/ui/reveal/RevealPathButton";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatDuration } from "@/lib/format/duration";
import { formatBytesPair } from "@/lib/memory/format";

interface IArchivesPackResultProps extends BaseComponentProps {
  result: ArchivePackResult;
}

export function ArchivesPackResult({
  "data-testid": dataTestId = "archives-pack-result",
  id,
  className,
  result,
}: IArchivesPackResultProps): ReactElement {
  const columns: Array<GridColDef> = useMemo(
    () => [{ field: "volume", headerName: "Volume", flex: 1, minWidth: 320, cellClassName: "monospace" }],
    []
  );

  const rows: Array<{ volume: string }> = useMemo(() => result.volumes.map((volume) => ({ volume })), [result.volumes]);

  const stats: Array<ICommandResultStat> = useMemo(() => {
    // One unit for both sizes, so the compression ratio stays readable at a glance.
    const [sizeSource, sizeWritten] = formatBytesPair(result.sizeSource, result.sizeWritten);

    return [
      { label: "volumes", value: result.volumes.length },
      { label: "packed", value: result.filesTotal },
      { label: "compressed", value: result.filesCompressed },
      { label: "stored", value: result.filesStored },
      // Aliased and skipped are the two counts that explain a surprising size or a missing file.
      { label: "aliased", value: result.filesAliased },
      { label: "skipped", value: result.filesSkipped },
      { label: "source", value: sizeSource },
      { label: "written", value: sizeWritten },
      { label: "elapsed", value: formatDuration(result.duration) },
    ];
  }, [result]);

  return (
    <CommandResult
      data-testid={dataTestId}
      id={id}
      className={className}
      headline={`Packed ${result.filesTotal} file(s) into ${result.volumes.length} volume(s)`}
      tone={"success"}
      stats={stats}
      // The first volume rather than the output directory, so the file manager opens with something
      // this run produced selected rather than with whatever else lives there.
      actions={
        <RevealPathButton
          application={EApplicationId.ARCHIVES_PACKER}
          path={result.volumes[0] ?? null}
          label={"Show volumes"}
        />
      }
    >
      <CommandResultFindings<{ volume: string }>
        rows={rows}
        columns={columns}
        getRowId={(row) => row.volume}
        getSearchText={(row) => row.volume}
        emptyLabel={"No volumes were written."}
        searchPlaceholder={"Filter by volume"}
      />
    </CommandResult>
  );
}
