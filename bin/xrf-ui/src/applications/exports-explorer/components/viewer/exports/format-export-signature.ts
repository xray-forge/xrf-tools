import { ExportDescriptor, ExportParameterDescriptor } from "@/core/bindings/types/xrf-export";

export function formatExportSignature(declaration: ExportDescriptor): string {
  switch (declaration.kind) {
    case "callable":
      return `${declaration.name}(${declaration.parameters
        .map(
          (parameter: ExportParameterDescriptor) =>
            `${parameter.name}${parameter.isOptional ? "?" : ""}: ${parameter.typing}`
        )
        .join(", ")}): ${declaration.returns.typing}`;

    case "value":
      return `${declaration.name}: ${declaration.typing}`;

    default: {
      return declaration as never;
    }
  }
}
