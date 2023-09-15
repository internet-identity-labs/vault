import { IOType, spawnSync } from "child_process";
import { readFileSync } from "fs";

const path: string = "test/resource";

export const sleep = async (seconds: number): Promise<void> => {
    await new Promise(resolve => setTimeout(resolve, seconds * 1000));
}

export const call = (command: string, stdio?: IOType): string => {
    var result = spawnSync(command, {
        stdio: stdio || "pipe",
        shell: true,
        encoding: 'utf-8'
    });

    if(result.status !== 0) {
        console.error(JSON.stringify(result));
    }

    return result.stdout ? result.stdout?.trim() : result.stderr?.trim();
};

export const execute = (command: string, stdio?: IOType): string => {
    console.debug("> " + command);
    return call(command, "inherit");
}

export const getFile = (file: string, ...params: string[]): string => {
    var content = readFileSync(path + file).toString().trim();

    params.forEach((x: string) => {
        content = content.replace("${s}", x);
    });

    return content;
}