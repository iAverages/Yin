import { execSync as exec } from "child_process";
import { Octokit } from "octokit";

// Below is passed in though the CI env.
const SERVICE_NAME = process.env.PACKAGE_NAME;
const GIT_HASH = process.env.BUILD_VCS_NUMBER;

const title = `Update ${SERVICE_NAME} to ${GIT_HASH}`;
const body = `This is an automated PR to update the ${SERVICE_NAME} service to the latest version.`;
const head = `update-${SERVICE_NAME}-${GIT_HASH}`;

const owner = "iAverages";
const repo = "Yin";
const base = "main";
const commitUser = {
    name: "kirsi-bot",
    email: "137784701+kirsi-bot@users.noreply.github.com",
};

const octokit = new Octokit({ auth: process.env.GITHUB_TOKEN });
const createPullRequest = async () => {
    return octokit.rest.pulls.create({
        owner,
        repo,
        title,
        head,
        body,
        base,
    });
};

const mergePullRequest = async (pr: number) => {
    return octokit.rest.pulls.merge({
        owner,
        repo,
        pull_number: pr,
        merge_method: "squash",
    });
};

const main = async () => {
    console.log(`Preparing local branch ${head}`);
    exec(`git config --global user.name "${commitUser.name}"`);
    exec(`git config --global user.email "${commitUser.email}"`);
    exec(`git checkout -b ${head}`);
    console.log(`Updating manifest to ${GIT_HASH}`);
    exec(
        `sed -i 's/image:.*/image: ctr.avrg.dev\\/yin\\/${SERVICE_NAME}:${GIT_HASH}/' configs\\/${SERVICE_NAME}\\/deployment.yaml`
    );
    console.log(`Commiting changes...`);
    exec(`git commit -am "${title}"`);
    console.log(`Pushing changes...`);
    exec(`git push --set-upstream origin ${head}`);
    console.log(`Creating PR...`);

    const pr = await createPullRequest();
    console.log(`Merging PR...`);
    await mergePullRequest(pr.data.number);
    console.log(`Done!`);
};

main();
