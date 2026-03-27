import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { createDefaultConfig } from "../lib/default-config";
import { ConfigPanel } from "./config-panel";

describe("ConfigPanel", () => {
  it("adds a model entry", async () => {
    const user = userEvent.setup();
    const current = createDefaultConfig();
    const onChange = vi.fn();

    render(
      <ConfigPanel
        config={current}
        runtimePythonPath="/tmp/runtime/.venv/bin/python"
        saveState="idle"
        onChange={onChange}
        onSave={async () => undefined}
      />,
    );

    await user.click(screen.getByRole("button", { name: /新增模型/ }));

    expect(onChange).toHaveBeenCalled();
    expect(
      screen.getByDisplayValue("/tmp/runtime/.venv/bin/python"),
    ).toHaveAttribute("readOnly");
    const latestConfig = onChange.mock.calls.at(-1)?.[0];
    expect(latestConfig.providerGroups).toHaveLength(1);
    expect(latestConfig.providerGroups[0].models).toHaveLength(2);
  });
});
