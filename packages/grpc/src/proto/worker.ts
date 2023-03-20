/* eslint-disable */
import {
  CallOptions,
  ChannelCredentials,
  Client,
  ClientOptions,
  ClientUnaryCall,
  handleUnaryCall,
  makeGenericClientConstructor,
  Metadata,
  ServiceError,
  UntypedServiceImplementation,
} from "@grpc/grpc-js";
import _m0 from "protobufjs/minimal.js";

export const protobufPackage = "worker";

export interface Packet {
  readonly body: string;
}

export interface StatusReply {
  readonly success: boolean;
  readonly message?: string | undefined;
}

function createBasePacket(): Packet {
  return { body: "" };
}

export const Packet = {
  encode(message: Packet, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.body !== "") {
      writer.uint32(10).string(message.body);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): Packet {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBasePacket() as any;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.body = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): Packet {
    return { body: isSet(object.body) ? String(object.body) : "" };
  },

  toJSON(message: Packet): unknown {
    const obj: any = {};
    message.body !== undefined && (obj.body = message.body);
    return obj;
  },

  create<I extends Exact<DeepPartial<Packet>, I>>(base?: I): Packet {
    return Packet.fromPartial(base ?? {});
  },

  fromPartial<I extends Exact<DeepPartial<Packet>, I>>(object: I): Packet {
    const message = createBasePacket() as any;
    message.body = object.body ?? "";
    return message;
  },
};

function createBaseStatusReply(): StatusReply {
  return { success: false, message: undefined };
}

export const StatusReply = {
  encode(message: StatusReply, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.success === true) {
      writer.uint32(8).bool(message.success);
    }
    if (message.message !== undefined) {
      writer.uint32(18).string(message.message);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): StatusReply {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseStatusReply() as any;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.success = reader.bool();
          break;
        case 2:
          message.message = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): StatusReply {
    return {
      success: isSet(object.success) ? Boolean(object.success) : false,
      message: isSet(object.message) ? String(object.message) : undefined,
    };
  },

  toJSON(message: StatusReply): unknown {
    const obj: any = {};
    message.success !== undefined && (obj.success = message.success);
    message.message !== undefined && (obj.message = message.message);
    return obj;
  },

  create<I extends Exact<DeepPartial<StatusReply>, I>>(base?: I): StatusReply {
    return StatusReply.fromPartial(base ?? {});
  },

  fromPartial<I extends Exact<DeepPartial<StatusReply>, I>>(object: I): StatusReply {
    const message = createBaseStatusReply() as any;
    message.success = object.success ?? false;
    message.message = object.message ?? undefined;
    return message;
  },
};

export type WorkerService = typeof WorkerService;
export const WorkerService = {
  handlePacket: {
    path: "/worker.Worker/handlePacket",
    requestStream: false,
    responseStream: false,
    requestSerialize: (value: Packet) => Buffer.from(Packet.encode(value).finish()),
    requestDeserialize: (value: Buffer) => Packet.decode(value),
    responseSerialize: (value: StatusReply) => Buffer.from(StatusReply.encode(value).finish()),
    responseDeserialize: (value: Buffer) => StatusReply.decode(value),
  },
} as const;

export interface WorkerServer extends UntypedServiceImplementation {
  handlePacket: handleUnaryCall<Packet, StatusReply>;
}

export interface WorkerClient extends Client {
  handlePacket(request: Packet, callback: (error: ServiceError | null, response: StatusReply) => void): ClientUnaryCall;
  handlePacket(
    request: Packet,
    metadata: Metadata,
    callback: (error: ServiceError | null, response: StatusReply) => void,
  ): ClientUnaryCall;
  handlePacket(
    request: Packet,
    metadata: Metadata,
    options: Partial<CallOptions>,
    callback: (error: ServiceError | null, response: StatusReply) => void,
  ): ClientUnaryCall;
}

export const WorkerClient = makeGenericClientConstructor(WorkerService, "worker.Worker") as unknown as {
  new (address: string, credentials: ChannelCredentials, options?: Partial<ClientOptions>): WorkerClient;
  service: typeof WorkerService;
};

type Builtin = Date | Function | Uint8Array | string | number | boolean | undefined;

export type DeepPartial<T> = T extends Builtin ? T
  : T extends Array<infer U> ? Array<DeepPartial<U>> : T extends ReadonlyArray<infer U> ? ReadonlyArray<DeepPartial<U>>
  : T extends {} ? { [K in keyof T]?: DeepPartial<T[K]> }
  : Partial<T>;

type KeysOfUnion<T> = T extends T ? keyof T : never;
export type Exact<P, I extends P> = P extends Builtin ? P
  : P & { [K in keyof P]: Exact<P[K], I[K]> } & { [K in Exclude<keyof I, KeysOfUnion<P>>]: never };

function isSet(value: any): boolean {
  return value !== null && value !== undefined;
}
