import api from "./index";

try {
    console.log(await api.guilds.auditLogs.get({ "guild.id": "849562494447124500", queryParams: { action_type: 24 } }));
} catch (e) {
    console.log(e);
}
