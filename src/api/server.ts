import express from "express";
import log from "@iaverage/logger";
const app = express();

app.listen(process.env.API_PORT, () => log.success(`API listen on port ${process.env.API_PORT}`));
