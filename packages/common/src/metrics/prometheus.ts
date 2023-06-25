import http from "http";
import prometheus from "prom-client";

prometheus.collectDefaultMetrics();

export const createPrometheusServer = () => {
    const server = http.createServer(async (req, res) => {
        if (req.url === "/metrics") {
            res.end(await prometheus.register.metrics());
            return;
        }
        res.statusCode = 404;
        res.end("404 Not found");
    });

    return server;
};
