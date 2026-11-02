import { ArchiveReadResult } from "@xrf/ipc/types/xrf-archive";
import { ESyntaxLanguage } from "@xrf/syntax";
import { Nullable } from "@xrf/types";
import { ICodeLine, ICodeLineSource, toTextLineSource, VirtualizedLines } from "@xrf/ui/code/lines";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { cn } from "@xrf/ui/theme";
import { ReactElement, useCallback, useMemo, useState } from "react";

import { getSyntaxLanguage } from "@/core/syntax/lib/syntax-language";

/** A line the reader marked, and the document it was a line of. */
interface ISelectedLine {
  layout: object;
  number: number;
}

interface IArchiveCodePreviewProps extends BaseComponentProps {
  file: ArchiveReadResult;
}

export function ArchiveCodePreview({
  "data-testid": dataTestId = "archive-code-preview",
  id,
  className,
  file,
}: IArchiveCodePreviewProps): ReactElement {
  const [selected, setSelected] = useState<Nullable<ISelectedLine>>(null);

  const language: ESyntaxLanguage = useMemo(() => getSyntaxLanguage(file.name), [file.name]);

  const source: ICodeLineSource = useMemo(() => toTextLineSource(file.content, language), [file.content, language]);

  const onSelectLine = useCallback(
    (line: ICodeLine) => setSelected({ layout: source.layout, number: line.number }),
    [source]
  );

  return (
    <VirtualizedLines
      data-testid={dataTestId}
      ariaLabel={`Contents of ${file.name}`}
      className={cn("min-h-0 min-w-0 grow", className)}
      id={id}
      selectedLine={selected?.layout === source.layout ? selected.number : null}
      source={source}
      onSelectLine={onSelectLine}
    />
  );
}
