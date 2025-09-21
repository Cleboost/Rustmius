export interface EntryConfig {
  id: string;
  name: string;
}

export interface Server extends EntryConfig {
  ip: string;
  username?: string;
  keyID: number;
}

export interface Folder extends EntryConfig {
  contents: Array<Folder | Server>;
}

export type ServerConfig = Array<Folder | Server>;
