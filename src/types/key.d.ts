export interface KeyPair {
  id: number; // Unique identifier
  name: string; // Display name
  private: string; // Path to private key
  public?: string; // Path to public key (optional)
}

export type KeysConfig = KeyPair[];
