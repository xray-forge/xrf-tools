import { GridColDef, GridValidRowModel, GridValueGetter } from "@mui/x-data-grid";

/** Applied by the theme to render a cell in monospace. */
const MONOSPACE_CLASS: string = "monospace";

/** Vector components are read for magnitude and sign, not for their tail digits. */
const VECTOR_PRECISION: number = 2;

interface IVectorLike {
  x: number;
  y: number;
  z: number;
}

function isVectorLike(value: unknown): value is IVectorLike {
  return typeof value === "object" && value !== null && "x" in value && "y" in value && "z" in value;
}

/**
 * Creates a plain numeric or string column.
 *
 * @param field - Field name read from each row.
 * @param headerName - Label shown in the column header.
 * @param width - Optional width in pixels.
 * @returns Grid column definition.
 */
export function textColumn(field: string, headerName: string, width?: number): GridColDef {
  return width === undefined ? { field, headerName } : { field, headerName, width };
}

/**
 * Creates a monospace column for identifiers, paths, or sections.
 *
 * @param field - Field name read from each row.
 * @param headerName - Label shown in the column header.
 * @param width - Optional width in pixels.
 * @returns Grid column definition.
 */
export function identifierColumn(field: string, headerName: string, width?: number): GridColDef {
  return {
    ...textColumn(field, headerName, width),
    cellClassName: MONOSPACE_CLASS,
  };
}

/**
 * Creates a fixed-precision vector column.
 *
 * @param field - Field name read from each row.
 * @param headerName - Label shown in the column header.
 * @param width - Column width in pixels.
 * @returns Grid column definition that renders vector-like values as `x, y, z` and other values as empty.
 */
export function vectorColumn(field: string, headerName: string, width: number = 170): GridColDef {
  return {
    field,
    headerName,
    width,
    cellClassName: MONOSPACE_CLASS,
    sortable: false,
    valueGetter: ((value: unknown) =>
      isVectorLike(value)
        ? `${value.x.toFixed(VECTOR_PRECISION)}, ${value.y.toFixed(VECTOR_PRECISION)}, ${value.z.toFixed(
            VECTOR_PRECISION
          )}`
        : null) as GridValueGetter<GridValidRowModel>,
  };
}

/**
 * Creates a hexadecimal bit-field column.
 *
 * @param field - Field name read from each row.
 * @param headerName - Label shown in the column header.
 * @param width - Column width in pixels.
 * @returns Grid column definition that renders numeric values as hexadecimal and other values as empty.
 */
export function flagsColumn(field: string, headerName: string, width: number = 110): GridColDef {
  return {
    field,
    headerName,
    width,
    cellClassName: MONOSPACE_CLASS,
    valueGetter: ((value: unknown) =>
      typeof value === "number" ? `0x${value.toString(16).toUpperCase()}` : null) as GridValueGetter<GridValidRowModel>,
  };
}

/**
 * Creates a compact numeric-tuple column.
 *
 * @param field - Field name read from each row.
 * @param headerName - Label shown in the column header.
 * @param width - Column width in pixels.
 * @returns Grid column definition that renders arrays as comma-separated values and other values as empty.
 */
export function tupleColumn(field: string, headerName: string, width: number = 150): GridColDef {
  return {
    field,
    headerName,
    width,
    cellClassName: MONOSPACE_CLASS,
    sortable: false,
    valueGetter: ((value: unknown) =>
      Array.isArray(value) ? value.join(", ") : null) as GridValueGetter<GridValidRowModel>,
  };
}

/**
 * Creates a fixed-precision decimal column.
 *
 * @param field - Field name read from each row.
 * @param headerName - Label shown in the column header.
 * @param width - Column width in pixels.
 * @returns Grid column definition that renders numeric values at fixed precision and other values as empty.
 */
export function decimalColumn(field: string, headerName: string, width: number = 110): GridColDef {
  return {
    field,
    headerName,
    width,
    type: "number",
    align: "left",
    headerAlign: "left",
    cellClassName: MONOSPACE_CLASS,
    valueGetter: ((value: unknown) => (typeof value === "number" ? value : null)) as GridValueGetter<GridValidRowModel>,
    valueFormatter: (value: unknown) => (typeof value === "number" ? value.toFixed(VECTOR_PRECISION) : null),
  };
}
