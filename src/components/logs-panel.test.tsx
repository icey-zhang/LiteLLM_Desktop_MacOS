import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { LogsPanel } from "./logs-panel";

describe("LogsPanel", () => {
  it("filters logs and clears current view", async () => {
    const user = userEvent.setup();
    const onClear = vi.fn();

    render(
      <LogsPanel
        logs={[
          { stream: "stdout", line: "proxy started", timestamp: new Date().toISOString() },
          { stream: "stderr", line: "auth error", timestamp: new Date().toISOString() },
        ]}
        onClear={onClear}
      />,
    );

    await user.type(screen.getByLabelText("日志过滤"), "auth");
    expect(screen.getByText("auth error")).toBeInTheDocument();
    expect(screen.queryByText("proxy started")).not.toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "清空日志" }));
    expect(onClear).toHaveBeenCalledTimes(1);
  });
});
