import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Shell } from "./shell";
import { createDefaultConfig } from "../lib/default-config";
import type { ProxyStatus } from "../types/config";

const proxyStatus: ProxyStatus = {
  state: "stopped",
  pid: null,
  port: 4000,
  endpoint: "http://127.0.0.1:4000",
  healthy: false,
  lastError: null,
};

describe("Shell", () => {
  it("switches between all primary panels", async () => {
    const user = userEvent.setup();

    render(
      <Shell
        config={createDefaultConfig()}
        environment={null}
        runtimeStatus={null}
        proxyStatus={proxyStatus}
        logs={[]}
        testResult={null}
        saveState="idle"
        onConfigChange={() => undefined}
        onSave={async () => undefined}
        onStart={async () => undefined}
        onStop={async () => undefined}
        onRestart={async () => undefined}
        onRetestEnvironment={async () => undefined}
        onBootstrapRuntime={async () => undefined}
        onClearLogs={() => undefined}
        onRunTest={async () => undefined}
      />,
    );

    expect(screen.getByRole("heading", { name: "状态与环境" })).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: /配置/ }));
    expect(screen.getByRole("heading", { name: "全局设置" })).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: /日志/ }));
    expect(screen.getByRole("heading", { name: "实时输出" })).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: /请求测试/ }));
    expect(screen.getByRole("heading", { name: "发送测试请求" })).toBeInTheDocument();
  });
});
