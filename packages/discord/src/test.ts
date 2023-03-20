import api from "./index";

try {
    console.log(await api.user.getCurrentUser());
} catch (e) {
    console.log(e);
}
