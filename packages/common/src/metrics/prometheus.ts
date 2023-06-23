import http from "http";
import prometheus from "prom-client";

prometheus.collectDefaultMetrics();

const counter = new prometheus.Counter({ name: "my_counter", help: "my_counter_help" });

(async () => {
    counter.inc();
    console.log(await prometheus.register.metrics());
    counter.inc();
    console.log(await prometheus.register.metrics());
    counter.inc();
    console.log(await prometheus.register.metrics());
})();

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
