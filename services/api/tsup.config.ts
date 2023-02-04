import { defineConfig } from "tsup";

export default defineConfig({
    entry: ["src/**/*.ts", "!src/**/*.d.ts", "src/**/*.tsx", "!src/**/*.test.ts*"],
    skipNodeModulesBundle: true,
    format: ["esm"],
    splitting: true,
    dts: true,
    clean: true,
    sourcemap: true,
    minify: true,
    target: "node18",
    noExternal: ["@yin/db", "@yin/trpc", "@yin/common", "@yin/auth"],
});
