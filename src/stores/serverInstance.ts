import { defineStore } from "pinia";
import { ref } from "vue";
import Server from "@/class/Server";

export const useServerInstanceStore = defineStore("serverInstance", () => {
  const serverInstance = ref<Server[]>([]);

  /**
   * Get a server instance by its ID.
   * @param id - The ID of the server.
   * @returns A promise that resolves to the Server instance or null if not found.
   */
  function getServerInstance(id: string): Server | null {
    return serverInstance.value.find(
      (server: Server) => server.id === id || null,
    );
  }

  /**
   * Add a server instance to the store if it doesn't already exist.
   * @param server - The Server instance to add.
   * @returns A promise that resolves to true if the server was added, false if it already exists.
   */
  async function addServerInstance(server: Server): Promise<boolean> {
    if (!server || serverInstance.value.find((s) => s.id === server.id))
      return false;
    serverInstance.value.push(server);
    return true;
  }

  /**
   * Remove a server instance from the store by its ID.
   * @param id - The ID of the server to remove.
   * @returns A promise that resolves to true if the server was removed, false if it was not found.
   */
  async function removeServerInstance(id: string): Promise<boolean> {
    if (!getServerInstance(id)) return false;
    serverInstance.value = serverInstance.value.filter(
      (server) => server.id !== id,
    );
    return true;
  }

  /**

   */
  function listServerInstances(): Server[] {
    return serverInstance.value;
  }

  return {
    getServerInstance,
    addServerInstance,
    removeServerInstance,
    listServerInstances,
  };
});
