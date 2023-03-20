import api from "./index";

(async () => {
    try {
        const res = await api.user.getCurrentUserGuilds();
        if (res.success) {
            console.log(res.data);
        } else if (!res.success && res._zodError) {
            console.log(res._zod);
        } else {
            console.log(res);
        }
    } catch (e) {
        console.log(e);
    }
})();
