import { Options } from "tsup";

export const tsupConfig: Options = {
    entry: ["src/**/*.ts", "!src/**/*.d.ts", "src/**/*.tsx"],
    skipNodeModulesBundle: true,
    format: ["esm"],
    clean: true,
    sourcemap: true,
    target: "node18",
    noExternal: ["@yin/db", "@yin/trpc", "@yin/common", "@yin/auth"],
};
