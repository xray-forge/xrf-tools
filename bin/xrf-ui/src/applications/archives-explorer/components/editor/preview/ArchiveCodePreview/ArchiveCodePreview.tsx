import { ReactElement, useMemo } from "react";

import { ArchiveReadResult } from "@/core/ipc/types/xrf-archive";
import { CodeView } from "@/core/syntax/components/CodeView";
import { ESyntaxLanguage, getSyntaxLanguage } from "@/core/syntax/lib";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IArchiveCodePreviewProps extends BaseComponentProps {
  file: ArchiveReadResult;
}

export function ArchiveCodePreview({
  "data-testid": dataTestId = "archive-code-preview",
  id,
  className,
  file,
}: IArchiveCodePreviewProps): ReactElement {
  const language: ESyntaxLanguage = useMemo(() => getSyntaxLanguage(file.name), [file.name]);

  return (
    <CodeView
      data-testid={dataTestId}
      id={id}
      className={className}
      label={`Contents of ${file.name}`}
      content={file.content}
      language={language}
      sx={{ flexGrow: 1, minHeight: 0, backgroundColor: "background.default" }}
    />
  );
}
