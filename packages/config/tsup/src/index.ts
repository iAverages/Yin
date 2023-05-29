import { type Options } from "tsup";

export const tsConfig: Options = {
    entry: ["src/**/*.ts", "!src/**/*.d.ts", "src/**/*.tsx", "!src/**/*.test.ts*"],
    skipNodeModulesBundle: true,
    format: ["esm"],
    clean: true,
    sourcemap: true,
    minify: true,
    target: "node18",
    noExternal: ["@yin/db", "@yin/trpc", "@yin/common", "@yin/auth", "@yin/grpc", "@yin/discord"],
    splitting: false,
};
