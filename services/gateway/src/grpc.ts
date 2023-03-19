import { WorkerClient } from "@yin/grpc";
import grpc from "@grpc/grpc-js";

const worker = new WorkerClient("localhost:50051", grpc.credentials.createInsecure());

export default worker;
