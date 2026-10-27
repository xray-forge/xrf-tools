import { describe, expect, it, jest } from "@jest/globals";
import { SxProps, Theme } from "@mui/material/styles";

import { mergeSx } from "./merge-sx";

describe("mergeSx", () => {
  it("preserves override order and defers theme callbacks to MUI", () => {
    const defaults = { display: "flex" };
    const override = { display: "grid" };
    const themed = jest.fn((theme: Theme) => ({ color: theme.palette.primary.main }));
    const styles: SxProps<Theme> = [false, override, themed];

    expect(mergeSx(defaults, undefined, styles)).toEqual([defaults, false, override, themed]);
    expect(themed).not.toHaveBeenCalled();
  });

  it("keeps nested styles intact and does not mutate readonly inputs", () => {
    const defaults = { "&:hover": { color: "primary.main" } };
    const override = { "&:hover": { opacity: 0.5 } };
    const styles = Object.freeze([override, null]);

    expect(mergeSx(defaults, styles)).toEqual([defaults, override, null]);
    expect(styles).toEqual([override, null]);
  });

  it("accepts absent optional styles", () => {
    expect(mergeSx()).toEqual([]);
    expect(mergeSx(undefined, undefined)).toEqual([]);
  });
});
