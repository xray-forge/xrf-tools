import { default as DataObjectIcon } from "@mui/icons-material/DataObject";
import { Nullable } from "@xrf/types";
import { ReactElement } from "react";

import { ExportDescriptor } from "@/core/ipc/types/xrf-export";
import { EmptyState } from "@/core/ui/layout/EmptyState";
import { BaseComponentProps } from "@/lib/dom/element-types";

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
