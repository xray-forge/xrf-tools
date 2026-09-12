import { ReactElement } from "react";

import { ExportDescriptor } from "@/core/bindings/types/xrf-export";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

import { ExportDeclarationView } from "./ExportDeclarationView";
import { ExportsViewerState } from "./ExportsViewerState";

export interface IExportsViewerProps extends BaseComponentProps {
  declaration: Nullable<ExportDescriptor>;
  exportCount: number;
}

export function ExportsViewer({ declaration, exportCount }: IExportsViewerProps): ReactElement {
  if (!exportCount) {
    return (
      <ExportsViewerState
        title={"No externs found"}
        description={"This project is open, but it does not currently declare any externs."}
      />
    );
  }

  if (!declaration) {
    return (
      <ExportsViewerState
        title={"Select an export to inspect"}
        description={"Expand a namespace in the explorer and select one of its declarations."}
      />
    );
  }

  return <ExportDeclarationView declaration={declaration} />;
}
