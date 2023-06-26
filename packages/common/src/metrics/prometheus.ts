import http from "http";
import prometheus from "prom-client";

prometheus.collectDefaultMetrics();

export const createPrometheusServer = (props?: { additional?: string[] }) => {
    const additional = props?.additional ?? [];
    const server = http.createServer(async (req, res) => {
        if (req.url === "/metrics") {
            res.setHeader("Content-Type", prometheus.register.contentType);
            res.end((await prometheus.register.metrics()) + additional.join("\n"));
            return;
        }
        res.statusCode = 404;
        res.end("404 Not found");
    });

    return server;
};
