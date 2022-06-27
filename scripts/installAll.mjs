import { execSync } from "child_process";
import ora from "ora";

const execJson = (command) => {
    const res = execSync(command);
    try {
        return JSON.parse(Buffer.from(res).toString());
    } catch (e) {
        console.log("Failed to get workspace information, is the workspace setup correctly?");
        process.exit(1);
    }
};

const args = process.argv.splice(2);
if (args.length < 1) {
    console.log("Please info something");
    process.exit(1);
}
const packages = execJson("yarn -s workspaces info");
console.log(`Found ${Object.keys(packages).length} packages`);

for (const name of Object.keys(packages)) {
    const spinner = ora(`Adding packages to ${name}`).start();
    execSync(`yarn workspace ${name} add ${args.join(" ")}`);
    spinner.succeed();
}
