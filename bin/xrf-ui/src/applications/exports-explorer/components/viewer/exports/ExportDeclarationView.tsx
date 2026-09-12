import { default as DataObjectIcon } from "@mui/icons-material/DataObject";
import { Box, Chip, Typography } from "@mui/material";
import { ReactElement } from "react";

import { ExportDescriptor } from "@/core/bindings/types/xrf-export";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { CallableExportDetails } from "./CallableExportDetails";
import { ExportSection } from "./ExportSection";
import { ExportSourceView } from "./ExportSourceView";
import { formatExportSignature } from "./format-export-signature";

export interface IExportDeclarationViewProps extends BaseComponentProps {
  declaration: ExportDescriptor;
}

export function ExportDeclarationView({ declaration }: IExportDeclarationViewProps): ReactElement {
  return (
    <Box sx={{ display: "flex", flexDirection: "column", flexGrow: 1, minWidth: 0, minHeight: 0 }}>
      <Box
        sx={{
          display: "flex",
          alignItems: "center",
          gap: 1,
          minHeight: 40,
          paddingX: 1.5,
          borderBottom: 1,
          borderColor: "divider",
          backgroundColor: "background.paper",
        }}
      >
        <DataObjectIcon fontSize={"small"} sx={{ color: "text.secondary" }} />
        <Typography noWrap variant={"body2"} className={"monospace"} sx={{ flexGrow: 1, minWidth: 0 }}>
          {declaration.name}
        </Typography>
        <Chip size={"small"} variant={"outlined"} label={declaration.kind === "callable" ? "Callable" : "Value"} />
      </Box>

      <Box sx={{ flexGrow: 1, minHeight: 0, overflowY: "auto", padding: 3 }}>
        <Box sx={{ width: "100%", maxWidth: 1440 }}>
          <ExportSection title={"Signature"}>
            <Typography
              component={"pre"}
              className={"monospace"}
              sx={{ margin: 0, whiteSpace: "pre-wrap", overflowWrap: "anywhere" }}
            >
              {formatExportSignature(declaration)}
            </Typography>
          </ExportSection>

          {declaration.description ? (
            <ExportSection title={"Description"}>
              <Typography variant={"body2"} sx={{ whiteSpace: "pre-wrap" }}>
                {declaration.description}
              </Typography>
            </ExportSection>
          ) : null}

          {declaration.kind === "callable" ? (
            <CallableExportDetails declaration={declaration} />
          ) : (
            <ExportSection title={"Value type"}>
              <Typography variant={"body2"} className={"monospace"} sx={{ overflowWrap: "anywhere" }}>
                {declaration.typing}
              </Typography>
            </ExportSection>
          )}

          <ExportSection title={"Source"} isLast={true}>
            <Typography
              className={"monospace"}
              variant={"body2"}
              sx={{ marginBottom: 1, color: "text.secondary", overflowWrap: "anywhere" }}
            >
              {declaration.source.path}:{declaration.source.line}:{declaration.source.column}
            </Typography>

            <ExportSourceView key={declaration.name} name={declaration.name} />
          </ExportSection>
        </Box>
      </Box>
    </Box>
  );
}
