import { execSync } from "child_process";

const tsProtoOptions = {
    env: "node",
    useReadonlyTypes: true,
    useMapType: true,
    importSuffix: ".js",
    esModuleInterop: true,
    outputServices: "grpc-js",
};

const tsConfig = [
    "--plugin=protoc-gen-ts=../../node_modules/.bin/protoc-gen-ts_proto",
    `--ts_proto_out=${Object.entries(tsProtoOptions).map(([k, v]) => `${k}=${v},`)}:./src/proto`,
    "-I ./proto ",
    "proto/*.proto",
];

// These extrat spaces make it look nicer in the console, at least for me
console.log("✨  Generating proto files");
execSync(`npx grpc_tools_node_protoc ${tsConfig.join(" ")}`);
console.log("✔️   Generated proto files");
