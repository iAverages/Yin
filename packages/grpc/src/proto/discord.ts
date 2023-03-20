/* eslint-disable */
import _m0 from "protobufjs/minimal.js";
import { Timestamp } from "./google/protobuf/timestamp.js";

export const protobufPackage = "discord";

export interface Message {
  readonly id: string;
  readonly channelId: string;
  readonly author: User | undefined;
  readonly content?: string | undefined;
  readonly timestamp: Date | undefined;
  readonly editedTimestamp?: Date | undefined;
}

export interface User {
  readonly id: string;
  readonly username: string;
}

function createBaseMessage(): Message {
  return {
    id: "",
    channelId: "",
    author: undefined,
    content: undefined,
    timestamp: undefined,
    editedTimestamp: undefined,
  };
}

export const Message = {
  encode(message: Message, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.id !== "") {
      writer.uint32(10).string(message.id);
    }
    if (message.channelId !== "") {
      writer.uint32(18).string(message.channelId);
    }
    if (message.author !== undefined) {
      User.encode(message.author, writer.uint32(26).fork()).ldelim();
    }
    if (message.content !== undefined) {
      writer.uint32(34).string(message.content);
    }
    if (message.timestamp !== undefined) {
      Timestamp.encode(toTimestamp(message.timestamp), writer.uint32(42).fork()).ldelim();
    }
    if (message.editedTimestamp !== undefined) {
      Timestamp.encode(toTimestamp(message.editedTimestamp), writer.uint32(50).fork()).ldelim();
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): Message {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseMessage() as any;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.id = reader.string();
          break;
        case 2:
          message.channelId = reader.string();
          break;
        case 3:
          message.author = User.decode(reader, reader.uint32());
          break;
        case 4:
          message.content = reader.string();
          break;
        case 5:
          message.timestamp = fromTimestamp(Timestamp.decode(reader, reader.uint32()));
          break;
        case 6:
          message.editedTimestamp = fromTimestamp(Timestamp.decode(reader, reader.uint32()));
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): Message {
    return {
      id: isSet(object.id) ? String(object.id) : "",
      channelId: isSet(object.channelId) ? String(object.channelId) : "",
      author: isSet(object.author) ? User.fromJSON(object.author) : undefined,
      content: isSet(object.content) ? String(object.content) : undefined,
      timestamp: isSet(object.timestamp) ? fromJsonTimestamp(object.timestamp) : undefined,
      editedTimestamp: isSet(object.editedTimestamp) ? fromJsonTimestamp(object.editedTimestamp) : undefined,
    };
  },

  toJSON(message: Message): unknown {
    const obj: any = {};
    message.id !== undefined && (obj.id = message.id);
    message.channelId !== undefined && (obj.channelId = message.channelId);
    message.author !== undefined && (obj.author = message.author ? User.toJSON(message.author) : undefined);
    message.content !== undefined && (obj.content = message.content);
    message.timestamp !== undefined && (obj.timestamp = message.timestamp.toISOString());
    message.editedTimestamp !== undefined && (obj.editedTimestamp = message.editedTimestamp.toISOString());
    return obj;
  },

  create<I extends Exact<DeepPartial<Message>, I>>(base?: I): Message {
    return Message.fromPartial(base ?? {});
  },

  fromPartial<I extends Exact<DeepPartial<Message>, I>>(object: I): Message {
    const message = createBaseMessage() as any;
    message.id = object.id ?? "";
    message.channelId = object.channelId ?? "";
    message.author = (object.author !== undefined && object.author !== null)
      ? User.fromPartial(object.author)
      : undefined;
    message.content = object.content ?? undefined;
    message.timestamp = object.timestamp ?? undefined;
    message.editedTimestamp = object.editedTimestamp ?? undefined;
    return message;
  },
};

function createBaseUser(): User {
  return { id: "", username: "" };
}

export const User = {
  encode(message: User, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.id !== "") {
      writer.uint32(10).string(message.id);
    }
    if (message.username !== "") {
      writer.uint32(18).string(message.username);
    }
    return writer;
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): User {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input);
    let end = length === undefined ? reader.len : reader.pos + length;
    const message = createBaseUser() as any;
    while (reader.pos < end) {
      const tag = reader.uint32();
      switch (tag >>> 3) {
        case 1:
          message.id = reader.string();
          break;
        case 2:
          message.username = reader.string();
          break;
        default:
          reader.skipType(tag & 7);
          break;
      }
    }
    return message;
  },

  fromJSON(object: any): User {
    return {
      id: isSet(object.id) ? String(object.id) : "",
      username: isSet(object.username) ? String(object.username) : "",
    };
  },

  toJSON(message: User): unknown {
    const obj: any = {};
    message.id !== undefined && (obj.id = message.id);
    message.username !== undefined && (obj.username = message.username);
    return obj;
  },

  create<I extends Exact<DeepPartial<User>, I>>(base?: I): User {
    return User.fromPartial(base ?? {});
  },

  fromPartial<I extends Exact<DeepPartial<User>, I>>(object: I): User {
    const message = createBaseUser() as any;
    message.id = object.id ?? "";
    message.username = object.username ?? "";
    return message;
  },
};

type Builtin = Date | Function | Uint8Array | string | number | boolean | undefined;

export type DeepPartial<T> = T extends Builtin ? T
  : T extends Array<infer U> ? Array<DeepPartial<U>> : T extends ReadonlyArray<infer U> ? ReadonlyArray<DeepPartial<U>>
  : T extends {} ? { [K in keyof T]?: DeepPartial<T[K]> }
  : Partial<T>;

type KeysOfUnion<T> = T extends T ? keyof T : never;
export type Exact<P, I extends P> = P extends Builtin ? P
  : P & { [K in keyof P]: Exact<P[K], I[K]> } & { [K in Exclude<keyof I, KeysOfUnion<P>>]: never };

function toTimestamp(date: Date): Timestamp {
  const seconds = date.getTime() / 1_000;
  const nanos = (date.getTime() % 1_000) * 1_000_000;
  return { seconds, nanos };
}

function fromTimestamp(t: Timestamp): Date {
  let millis = t.seconds * 1_000;
  millis += t.nanos / 1_000_000;
  return new Date(millis);
}

function fromJsonTimestamp(o: any): Date {
  if (o instanceof Date) {
    return o;
  } else if (typeof o === "string") {
    return new Date(o);
  } else {
    return fromTimestamp(Timestamp.fromJSON(o));
  }
}

function isSet(value: any): boolean {
  return value !== null && value !== undefined;
}
