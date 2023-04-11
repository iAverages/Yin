import api from "./index";
import { responseTypes } from "./manager";

(async () => {
    try {
        const res = await api.user.getCurrentUserGuilds();

        switch (res.type) {
            case responseTypes.SUCCESS:
                console.log(res.data);
                break;
            case responseTypes.DISCORD_ERROR:
                console.log(res.errors);
                break;
            case responseTypes.VALIDATION_ERROR:
                console.log(res.issues);
                break;
            case responseTypes.UNKNOWN_ERROR:
                console.log(res.message);
                break;
        }
    } catch (e) {
        console.log(e);
    }
})();
