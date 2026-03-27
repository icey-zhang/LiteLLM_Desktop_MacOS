import { createDefaultConfig, createModelEntry } from "./default-config";

describe("default config", () => {
  it("creates a usable desktop config skeleton", () => {
    const config = createDefaultConfig();

    expect(config.settings.pythonPath).toBe("python3");
    expect(config.settings.port).toBe(4000);
    expect(config.providerGroups).toHaveLength(1);
    expect(config.providerGroups[0].models).toHaveLength(1);
  });

  it("creates model entries without provider-specific secrets", () => {
    const first = createModelEntry();
    const second = createModelEntry();

    expect(first.alias).toBeTruthy();
    expect(first.id).not.toBe(second.id);
    expect("provider" in first).toBe(false);
  });
});
