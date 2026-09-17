import { default as DataObjectIcon } from "@mui/icons-material/DataObject";
import { Chip, Typography } from "@mui/material";
import { ReactElement } from "react";

import { ExportDescriptor } from "@/core/ipc/types/xrf-export";
import { EditorFileHeader } from "@/core/shell/editor/EditorFileHeader";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { CallableExportDetails } from "./CallableExportDetails";
import { ExportSection } from "./ExportSection";
import { ExportSourceView } from "./ExportSourceView";
import { formatExportSignature } from "./format-export-signature";

export interface IExportDeclarationViewProps extends BaseComponentProps {
  declaration: ExportDescriptor;
  /** Ends the selection without closing the project, which the header offers. */
  onDeselect: () => void;
}

export function ExportDeclarationView({ declaration, onDeselect }: IExportDeclarationViewProps): ReactElement {
  return (
    <div className={"flex min-h-0 min-w-0 grow flex-col"}>
      <EditorFileHeader
        data-testid={"export-declaration-header"}
        name={declaration.name}
        icon={<DataObjectIcon fontSize={"small"} className={"text-text-secondary"} />}
        actions={
          <Chip size={"small"} variant={"outlined"} label={declaration.kind === "callable" ? "Callable" : "Value"} />
        }
        closeLabel={"Close declaration"}
        closeDescription={"Clear the selection and close this declaration"}
        onClose={onDeselect}
      />

      <div className={"min-h-0 grow overflow-y-auto p-6"}>
        <div className={"w-full max-w-360"}>
          <ExportSection title={"Signature"}>
            <Typography component={"pre"} className={"monospace m-0 wrap-anywhere whitespace-pre-wrap"}>
              {formatExportSignature(declaration)}
            </Typography>
          </ExportSection>

          {declaration.description ? (
            <ExportSection title={"Description"}>
              <Typography className={"whitespace-pre-wrap"} variant={"body2"}>
                {declaration.description}
              </Typography>
            </ExportSection>
          ) : null}

          {declaration.kind === "callable" ? (
            <CallableExportDetails declaration={declaration} />
          ) : (
            <ExportSection title={"Value type"}>
              <Typography className={"monospace wrap-anywhere"} variant={"body2"}>
                {declaration.typing}
              </Typography>
            </ExportSection>
          )}

          <ExportSection title={"Source"} isLast={true}>
            <Typography className={"monospace mb-2 wrap-anywhere text-text-secondary"} variant={"body2"}>
              {declaration.source.path}:{declaration.source.line}:{declaration.source.column}
            </Typography>

            <ExportSourceView key={declaration.name} name={declaration.name} />
          </ExportSection>
        </div>
      </div>
    </div>
  );
}
