import { useKeysStore } from "@/stores/keys";
import { KeyPair } from "@/types/key";
import { homeDir } from "@tauri-apps/api/path";

export default class Key {
    readonly id: number;
    readonly keysStore = useKeysStore();
    readonly conf: KeyPair

    constructor(id: number) {
        this.id = id;
        this.conf = this.keysStore.getKey(id);
    }

    getID(): number {
        return this.id;
    }

    getName(): string {
        return this.conf?.name || "N/A";
    }

    async getPath(): Promise<string> {
        return `${await homeDir()}/.ssh/${this.conf?.name}`;
    } 
}