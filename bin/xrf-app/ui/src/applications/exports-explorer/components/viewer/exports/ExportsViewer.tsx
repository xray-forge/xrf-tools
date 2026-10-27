import { ExportDescriptor } from "@xrf/ipc/types/xrf-export";
import { Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { DataObjectIcon } from "@xrf/ui/icons";
import { ReactElement } from "react";

import { EmptyState } from "@/core/ui/layout/EmptyState";

import { ExportDeclarationView } from "./ExportDeclarationView";

export interface IExportsViewerProps extends BaseComponentProps {
  declaration: Nullable<ExportDescriptor>;
  exportCount: number;
  /** Ends the selection without closing the project, which the header offers. */
  onDeselect: () => void;
}

export function ExportsViewer({ declaration, exportCount, onDeselect }: IExportsViewerProps): ReactElement {
  if (!exportCount) {
    return (
      <EmptyState
        icon={<DataObjectIcon className={"text-text-secondary opacity-55"} />}
        title={"No externs found"}
        description={"This project is open, but it does not currently declare any externs."}
      />
    );
  }

  if (!declaration) {
    return (
      <EmptyState
        icon={<DataObjectIcon className={"text-text-secondary opacity-55"} />}
        title={"Select an export to inspect"}
        description={"Expand a namespace in the explorer and select one of its declarations."}
      />
    );
  }

  return <ExportDeclarationView declaration={declaration} onDeselect={onDeselect} />;
}
