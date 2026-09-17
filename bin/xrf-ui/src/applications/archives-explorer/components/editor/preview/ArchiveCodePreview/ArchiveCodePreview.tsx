import { ReactElement, useCallback, useMemo, useState } from "react";

import { ArchiveReadResult } from "@/core/ipc/types/xrf-archive";
import { ESyntaxLanguage, getSyntaxLanguage } from "@/core/syntax/lib";
import { ICodeLine, ICodeLineSource } from "@/core/ui/code/code-line";
import { toTextLineSource } from "@/core/ui/code/text-line-source";
import { VirtualizedLines } from "@/core/ui/code/VirtualizedLines";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

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
