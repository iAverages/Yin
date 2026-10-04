import { createPool } from "mysql2/promise";

export function createDatabasePool(databaseUrl: string) {
  return createPool({ uri: databaseUrl, timezone: "Z", connectionLimit: 10 });
}
