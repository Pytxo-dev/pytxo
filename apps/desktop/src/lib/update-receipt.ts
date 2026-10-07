export type UpdateReceipt = { fromVersion: string; toVersion: string; requestedAt: string };
export interface UpdateReceiptStore {
  read(): UpdateReceipt | null;
  write(receipt: UpdateReceipt): void;
  clear(): void;
}

const KEY = "pytxo-desktop-update-request-v1";
export function browserUpdateReceipts(storage: Pick<Storage, "getItem" | "setItem" | "removeItem">): UpdateReceiptStore {
  return {
    read() {
      const raw = storage.getItem(KEY);
      if (!raw) return null;
      const receipt: unknown = JSON.parse(raw);
      if (!receipt || typeof receipt !== "object"
        || !("fromVersion" in receipt) || typeof receipt.fromVersion !== "string"
        || !("toVersion" in receipt) || typeof receipt.toVersion !== "string"
        || !("requestedAt" in receipt) || typeof receipt.requestedAt !== "string") {
        throw new Error("The saved update request is unreadable.");
      }
      return receipt as UpdateReceipt;
    },
    write: receipt => storage.setItem(KEY, JSON.stringify(receipt)),
    clear: () => storage.removeItem(KEY),
  };
}
