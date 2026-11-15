import { ExportDescriptor } from "@xrf/ipc/types/xrf-export";
import { Chip } from "@xrf/ui/badge";
import { EditorFileHeader } from "@xrf/ui/editor/file-header";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { DataObjectIcon } from "@xrf/ui/icons/data-object";
import { DetailSection } from "@xrf/ui/layout/detail-section";
import { Divider } from "@xrf/ui/layout/stack";
import { Typography } from "@xrf/ui/typography";
import { ReactElement } from "react";

import { CallableExportDetails } from "./CallableExportDetails";
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
          <DetailSection title={"Signature"}>
            <Typography component={"pre"} className={"monospace m-0 wrap-anywhere whitespace-pre-wrap"}>
              {formatExportSignature(declaration)}
            </Typography>
          </DetailSection>

          <Divider className={"my-5"} />

          {declaration.description ? (
            <>
              <DetailSection title={"Description"}>
                <Typography className={"whitespace-pre-wrap"} variant={"body2"}>
                  {declaration.description}
                </Typography>
              </DetailSection>

              <Divider className={"my-5"} />
            </>
          ) : null}

          {declaration.kind === "callable" ? (
            <CallableExportDetails declaration={declaration} />
          ) : (
            <DetailSection title={"Value type"}>
              <Typography className={"monospace wrap-anywhere"} variant={"body2"}>
                {declaration.typing}
              </Typography>
            </DetailSection>
          )}

          <Divider className={"my-5"} />

          <DetailSection title={"Source"}>
            <Typography className={"monospace mb-2 wrap-anywhere text-text-secondary"} variant={"body2"}>
              {declaration.source.path}:{declaration.source.line}:{declaration.source.column}
            </Typography>

            <ExportSourceView key={declaration.name} name={declaration.name} />
          </DetailSection>
        </div>
      </div>
    </div>
  );
}
