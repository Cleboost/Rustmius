<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import type { KeyPair } from "@/types/key";
import { useKeysStore } from "@/stores/keys";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter as DialogModalFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { toast } from "vue-sonner";
import {
  rename as fsRename,
  remove as fsRemove,
  exists as fsExists,
  mkdir as fsMkdir,
} from "@tauri-apps/plugin-fs";
import { dirname, join, homeDir } from "@tauri-apps/api/path";
import { Command } from "@tauri-apps/plugin-shell";
import {
  Loader2,
  Pencil,
  Plus,
  RefreshCw,
  Trash2,
  KeyRound,
} from "lucide-vue-next";

const keysStore = useKeysStore();

const loading = ref(true);
const refreshing = ref(false);

const renameDialogOpen = ref(false);
const renameTarget = ref<KeyPair | null>(null);
const renameValue = ref("");
const renameError = ref("");
const renaming = ref(false);

const deleteDialogOpen = ref(false);
const deleteTarget = ref<KeyPair | null>(null);
const deleting = ref(false);

const generateDialogOpen = ref(false);
const generateName = ref("id_rsa");
const generateLength = ref("4096");
const generateError = ref("");
const generating = ref(false);

const lengthOptions = ["2048", "3072", "4096", "8192"];

const keys = computed(() =>
  keysStore
    .listKeys()
    .slice()
    .sort((a, b) => a.name.localeCompare(b.name)),
);

function suggestKeyName(): string {
  const base = "id_rsa";
  const existing = new Set(keys.value.map((k) => k.name));
  if (!existing.has(base)) return base;
  let index = 1;
  while (existing.has(`${base}_${index}`)) {
    index += 1;
  }
  return `${base}_${index}`;
}

function resetGenerateForm(): void {
  generateName.value = suggestKeyName();
  generateLength.value = "4096";
  generateError.value = "";
}

async function refreshKeys(): Promise<void> {
  try {
    refreshing.value = true;
    await keysStore.syncWithFs();
  } catch (error) {
    console.error("sync keys", error);
    toast.error("Impossible d'actualiser les clés SSH.");
  } finally {
    refreshing.value = false;
    loading.value = false;
  }
}

onMounted(async () => {
  await refreshKeys();
});

function openRename(key: KeyPair): void {
  renameTarget.value = key;
  renameValue.value = key.name;
  renameError.value = "";
  renameDialogOpen.value = true;
}

function openDelete(key: KeyPair): void {
  deleteTarget.value = key;
  deleting.value = false;
  deleteDialogOpen.value = true;
}

function openGenerateModal(): void {
  resetGenerateForm();
  generateDialogOpen.value = true;
}

async function handleRename(): Promise<void> {
  if (!renameTarget.value) return;

  const newName = renameValue.value.trim();
  if (!newName) {
    renameError.value = "Le nom ne peut pas être vide.";
    return;
  }
  if (!/^[A-Za-z0-9._-]+$/.test(newName)) {
    renameError.value =
      "Le nom doit contenir uniquement lettres, chiffres, '.', '-' ou '_'.";
    return;
  }
  if (newName === renameTarget.value.name) {
    renameDialogOpen.value = false;
    return;
  }

  try {
    renaming.value = true;

    const privateDir = await dirname(renameTarget.value.private);
    const newPrivatePath = await join(privateDir, newName);

    if (newPrivatePath !== renameTarget.value.private) {
      const privateConflict = await fsExists(newPrivatePath).catch(() => false);
      if (privateConflict) {
        renameError.value = "Un fichier avec ce nom existe déjà.";
        renaming.value = false;
        return;
      }
    }

    let newPublicPath: string | undefined;
    if (renameTarget.value.public) {
      const publicDir = await dirname(renameTarget.value.public);
      newPublicPath = await join(publicDir, `${newName}.pub`);

      if (newPublicPath !== renameTarget.value.public) {
        const publicConflict = await fsExists(newPublicPath).catch(() => false);
        if (publicConflict) {
          renameError.value = "La clé publique existe déjà avec ce nom.";
          renaming.value = false;
          return;
        }
      }
    }

    if (newPrivatePath !== renameTarget.value.private) {
      await fsRename(renameTarget.value.private, newPrivatePath);
    }
    if (renameTarget.value.public && newPublicPath && newPublicPath !== renameTarget.value.public) {
      await fsRename(renameTarget.value.public, newPublicPath);
    }

    const update: Partial<Omit<KeyPair, "id">> = {
      name: newName,
      private: newPrivatePath,
    };
    if (renameTarget.value.public) update.public = newPublicPath;
    await keysStore.updateKeyMetadata(renameTarget.value.id, update);
    await keysStore.syncWithFs();

    toast.success("Clé renommée avec succès.");
    renameDialogOpen.value = false;
    renameTarget.value = null;
    renameValue.value = "";
  } catch (error) {
    console.error("rename key", error);
    renameError.value = "Échec du renommage de la clé.";
    toast.error("Échec du renommage de la clé.");
  } finally {
    renaming.value = false;
  }
}

async function handleDelete(): Promise<void> {
  if (!deleteTarget.value) return;

  try {
    deleting.value = true;

    if (deleteTarget.value.public) {
      const publicExists = await fsExists(deleteTarget.value.public).catch(
        () => false,
      );
      if (publicExists) {
        await fsRemove(deleteTarget.value.public).catch(() => undefined);
      }
    }

    const privateExists = await fsExists(deleteTarget.value.private).catch(
      () => false,
    );
    if (privateExists) {
      await fsRemove(deleteTarget.value.private).catch(() => undefined);
    }

    await keysStore.removeKey(deleteTarget.value.id);
    await keysStore.syncWithFs();

    toast.success("Clé supprimée.");
    deleteDialogOpen.value = false;
    deleteTarget.value = null;
  } catch (error) {
    console.error("delete key", error);
    toast.error("Impossible de supprimer la clé.");
  } finally {
    deleting.value = false;
  }
}

async function handleGenerate(): Promise<void> {
  const name = generateName.value.trim();
  if (!name) {
    generateError.value = "Le nom ne peut pas être vide.";
    return;
  }
  if (!/^[A-Za-z0-9._-]+$/.test(name)) {
    generateError.value =
      "Le nom doit contenir uniquement lettres, chiffres, '.', '-' ou '_'.";
    return;
  }

  generateError.value = "";

  try {
    generating.value = true;

    const home = await homeDir();
    const sshDir = await join(home, ".ssh");
    await fsMkdir(sshDir, { recursive: true }).catch(() => undefined);

    const privatePath = await join(sshDir, name);
    const publicPath = `${privatePath}.pub`;

    const privateExists = await fsExists(privatePath).catch(() => false);
    const publicExists = await fsExists(publicPath).catch(() => false);
    if (privateExists || publicExists) {
      generateError.value = "Une clé avec ce nom existe déjà.";
      generating.value = false;
      return;
    }

    const command = Command.create("ssh-keygen", [
      "-q",
      "-t",
      "rsa",
      "-b",
      generateLength.value,
      "-f",
      privatePath,
      "-N",
      "",
      "-C",
      name,
    ]);

    const result = await command.execute();
    if (result.code !== 0) {
      const message =
        result.stderr?.trim() || `ssh-keygen a échoué (code ${result.code}).`;
      throw new Error(message);
    }

    await keysStore.syncWithFs();
    await refreshKeys();

    toast.success("Clé SSH générée.");
    generateDialogOpen.value = false;
    resetGenerateForm();
  } catch (error) {
    console.error("generate key", error);
    const message = (error as Error)?.message ?? "Impossible de générer la clé.";
    generateError.value = message;
    toast.error("Impossible de générer la clé.");
  } finally {
    generating.value = false;
  }
}
</script>

<template>
  <div class="flex h-full flex-col gap-6">
    <div class="flex flex-wrap items-center justify-between gap-4">
      <div>
        <h1 class="text-3xl font-bold tracking-tight">Clés SSH</h1>
        <p class="text-muted-foreground">
          Gérez les clés présentes dans votre dossier <code>~/.ssh</code>.
        </p>
      </div>
      <div class="flex items-center gap-2">
        <Button
          variant="outline"
          size="sm"
          :disabled="refreshing"
          @click="refreshKeys"
        >
          <RefreshCw
            v-if="refreshing"
            class="mr-2 size-4 animate-spin"
          />
          <RefreshCw v-else class="mr-2 size-4" />
          Actualiser
        </Button>
        <Button size="sm" @click="openGenerateModal">
          <Plus class="mr-2 size-4" />
          Nouvelle clé
        </Button>
      </div>
    </div>

    <div class="flex-1 overflow-auto">
      <div
        v-if="loading"
        class="flex h-full items-center justify-center text-muted-foreground"
      >
        <Loader2 class="size-6 animate-spin" />
      </div>
      <div
        v-else-if="keys.length === 0"
        class="flex h-full flex-col items-center justify-center gap-3 text-center text-muted-foreground"
      >
        <KeyRound class="size-12" />
        <div class="max-w-sm space-y-1">
          <p class="text-lg font-semibold">Aucune clé détectée</p>
          <p class="text-sm">
            Utilisez le bouton « Nouvelle clé » pour en créer une ou ajoutez des
            fichiers dans <code>~/.ssh</code> puis actualisez.
          </p>
        </div>
        <Button size="sm" @click="openGenerateModal">
          <Plus class="mr-2 size-4" />
          Générer une clé
        </Button>
      </div>
      <div v-else class="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
        <Card v-for="key in keys" :key="key.id" class="flex flex-col">
          <CardHeader class="pb-2">
            <CardTitle class="flex items-center justify-between text-lg">
              <span>{{ key.name }}</span>
              <span class="text-xs font-medium text-muted-foreground">
                ID&nbsp;{{ key.id }}
              </span>
            </CardTitle>
          </CardHeader>
          <CardContent class="flex flex-col gap-3 text-sm">
            <div>
              <p class="text-muted-foreground text-xs uppercase">Privée</p>
              <p class="font-mono break-all text-xs">{{ key.private }}</p>
            </div>
            <div>
              <p class="text-muted-foreground text-xs uppercase">Publique</p>
              <p class="font-mono break-all text-xs">
                {{ key.public ?? "Non disponible" }}
              </p>
            </div>
          </CardContent>
          <CardFooter class="mt-auto flex justify-end gap-2">
            <Button
              size="sm"
              variant="outline"
              @click="openRename(key)"
            >
              <Pencil class="mr-2 size-4" />
              Renommer
            </Button>
            <Button
              size="sm"
              variant="destructive"
              @click="openDelete(key)"
            >
              <Trash2 class="mr-2 size-4" />
              Supprimer
            </Button>
          </CardFooter>
        </Card>
      </div>
    </div>

    <Dialog v-model:open="generateDialogOpen">
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Générer une clé SSH</DialogTitle>
          <DialogDescription>
            Créez une nouvelle paire RSA directement dans votre dossier
            <code>~/.ssh</code>.
          </DialogDescription>
        </DialogHeader>
        <div class="space-y-4 py-4">
          <div class="space-y-2">
            <Label for="generate-name">Nom du fichier</Label>
            <Input
              id="generate-name"
              v-model="generateName"
              :disabled="generating"
              placeholder="id_rsa"
            />
          </div>
          <div class="space-y-2">
            <Label for="generate-length">Longueur</Label>
            <Select
              id="generate-length"
              v-model="generateLength"
              :disabled="generating"
            >
              <SelectTrigger>
                <SelectValue placeholder="4096 bits" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem
                  v-for="length in lengthOptions"
                  :key="length"
                  :value="length"
                >
                  {{ length }} bits
                </SelectItem>
              </SelectContent>
            </Select>
          </div>
          <p v-if="generateError" class="text-sm text-destructive">
            {{ generateError }}
          </p>
          <p class="text-xs text-muted-foreground">
            La commande <code>ssh-keygen</code> sera exécutée avec une passphrase
            vide.
          </p>
        </div>
        <DialogModalFooter>
          <Button
            variant="outline"
            :disabled="generating"
            @click="generateDialogOpen = false"
          >
            Annuler
          </Button>
          <Button :disabled="generating" @click="handleGenerate">
            <Loader2 v-if="generating" class="mr-2 size-4 animate-spin" />
            Générer
          </Button>
        </DialogModalFooter>
      </DialogContent>
    </Dialog>

    <Dialog v-model:open="renameDialogOpen">
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Renommer la clé</DialogTitle>
          <DialogDescription>
            Donnez un nouveau nom à la paire. Les fichiers seront renommés sur votre disque.
          </DialogDescription>
        </DialogHeader>
        <div class="space-y-4 py-4">
          <div class="space-y-2">
            <Label for="rename-name">Nom de la clé</Label>
            <Input
              id="rename-name"
              v-model="renameValue"
              :disabled="renaming"
              placeholder="id_ed25519"
            />
            <p v-if="renameError" class="text-sm text-destructive">
              {{ renameError }}
            </p>
          </div>
          <p class="text-xs text-muted-foreground">
            Évitez d'utiliser des espaces ou des caractères spéciaux.
          </p>
        </div>
        <DialogModalFooter>
          <Button
            variant="outline"
            :disabled="renaming"
            @click="renameDialogOpen = false"
          >
            Annuler
          </Button>
          <Button :disabled="renaming" @click="handleRename">
            <Loader2 v-if="renaming" class="mr-2 size-4 animate-spin" />
            Enregistrer
          </Button>
        </DialogModalFooter>
      </DialogContent>
    </Dialog>

    <AlertDialog v-model:open="deleteDialogOpen">
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>Supprimer cette clé ?</AlertDialogTitle>
          <AlertDialogDescription>
            Cette action supprime définitivement les fichiers privés et publics
            associés. Assurez-vous de disposer d'une sauvegarde si nécessaire.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel :disabled="deleting">
            Annuler
          </AlertDialogCancel>
          <AlertDialogAction :disabled="deleting" @click="handleDelete">
            <Loader2
              v-if="deleting"
              class="mr-2 size-4 animate-spin"
            />
            Supprimer
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  </div>
</template>
