import Button from "../components/button";
import { trpc } from "../utils/trpc";

const Home = () => {
    const data = trpc.dev.getDevMessage.useQuery();
    return (
        <div>
            <div>Home page</div>
            <Button loading>hello</Button>
            <p>{data.data}</p>
            <p>{data.isLoading ? "true" : "false"}</p>
        </div>
    );
};

export default Home;
