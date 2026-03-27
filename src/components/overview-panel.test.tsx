import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { OverviewPanel } from "./overview-panel";

describe("OverviewPanel", () => {
  it("renders status and triggers restart", async () => {
    const user = userEvent.setup();
    const onRestart = vi.fn().mockResolvedValue(undefined);

    render(
      <OverviewPanel
        environment={{
          python: { ok: true, detail: "Python 3.14.3" },
          runtime: { ok: false, detail: "等待安装" },
          overallOk: false,
        }}
        runtimeStatus={{
          state: "ready",
          detail: "运行环境已就绪",
          pythonPath: "/tmp/runtime/.venv/bin/python",
          runtimeDir: "/tmp/runtime/.venv",
          version: "1.75.0",
          hint: null,
        }}
        proxyStatus={{
          state: "running",
          pid: 123,
          port: 4000,
          endpoint: "http://127.0.0.1:4000",
          healthy: true,
          lastError: null,
        }}
        onStart={async () => undefined}
        onStop={async () => undefined}
        onRestart={onRestart}
        onRetestEnvironment={async () => undefined}
        onBootstrapRuntime={async () => undefined}
      />,
    );

    expect(screen.getByText("运行中")).toBeInTheDocument();
    expect(screen.getByText("运行环境已就绪")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "启动代理" })).toHaveClass("btn", "btn-primary");
    expect(screen.getByRole("button", { name: "重启代理" })).toHaveClass("btn", "btn-secondary");
    expect(screen.getByRole("button", { name: "停止代理" })).toHaveClass(
      "btn",
      "btn-outline-danger",
    );
    expect(screen.getByRole("button", { name: "重新检测环境" })).toHaveClass("btn", "btn-secondary");
    expect(screen.getByRole("button", { name: "修复运行环境" })).toHaveClass("btn", "btn-secondary");

    await user.click(screen.getByRole("button", { name: "重启代理" }));
    expect(onRestart).toHaveBeenCalledTimes(1);
  });
});
