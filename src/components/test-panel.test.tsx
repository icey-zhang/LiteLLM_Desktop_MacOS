import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { createDefaultConfig } from "../lib/default-config";
import { TestPanel } from "./test-panel";

describe("TestPanel", () => {
  it("submits the current prompts", async () => {
    const user = userEvent.setup();
    const onRunTest = vi.fn().mockResolvedValue(undefined);
    const config = createDefaultConfig();

    render(
      <TestPanel
        models={config.providerGroups[0].models}
        port={config.settings.port}
        masterKey=""
        result={null}
        onRunTest={onRunTest}
      />,
    );

    await user.click(screen.getByRole("button", { name: "发送测试请求" }));
    expect(onRunTest).toHaveBeenCalledTimes(1);
    expect(onRunTest).toHaveBeenCalledWith(
      "gpt-4o-mini",
      "你是一个本地连通性测试助手。",
      "请回答：LiteLLM 代理已连接。",
    );
  });
});
