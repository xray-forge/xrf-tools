import { describe, expect, it } from "@jest/globals";
import { render } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { ReactElement } from "react";

import { useForceUpdate } from "@/lib/react/use-force-update";

let outside: number = 0;

function Component(): ReactElement {
  const forceUpdate: () => void = useForceUpdate();

  return (
    <button
      onClick={() => {
        outside += 1;

        forceUpdate();
      }}
    >
      {String(outside)}
    </button>
  );
}

describe("useForceUpdate", () => {
  it("renders again when something React cannot see has changed", async () => {
    outside = 0;

    const { getByRole } = render(<Component />);

    expect(getByRole("button").textContent).toBe("0");

    await userEvent.click(getByRole("button"));

    expect(getByRole("button").textContent).toBe("1");

    await userEvent.click(getByRole("button"));

    expect(getByRole("button").textContent).toBe("2");
  });

  it("keeps the same callback across renders, so an effect depending on it does not re-run", async () => {
    outside = 0;

    const seen: Set<() => void> = new Set();

    function Recording(): ReactElement {
      const forceUpdate: () => void = useForceUpdate();

      seen.add(forceUpdate);

      return <button onClick={forceUpdate}>{String(seen.size)}</button>;
    }

    const { getByRole } = render(<Recording />);

    await userEvent.click(getByRole("button"));

    expect(seen.size).toBe(1);
  });
});
