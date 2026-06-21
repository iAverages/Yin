import { createPool } from "mysql2/promise";

export function createDatabasePool(databaseUrl: string) {
  const url = new URL(databaseUrl);

  return createPool({
    host: url.hostname,
    port: url.port ? Number.parseInt(url.port, 10) : 3306,
    user: decodeURIComponent(url.username),
    password: decodeURIComponent(url.password),
    database: url.pathname.replace(/^\//, ""),
    timezone: "Z",
    connectionLimit: 10,
  });
}
