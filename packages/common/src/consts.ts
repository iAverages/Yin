export const consts = {
    version: "0.0.1",
    github: "https://github.com/iAverages/Yin",
    discord: {
        api: "https://discord.com/api/v10",
        cdn: "",
        gateway: "wss://gateway.discord.gg/?v=10&encoding=json",
    },
    rest: {
        get userAgent() {
            return `Yin Rest API Wrapper (${consts.github}), ${consts.version})`;
        },
    },
};
