export type QueueInsertOptions = {
  priority?: number;
};

type QueueEntry<T> = {
  value: T;
  priority: number;
  enqueuedAt: number;
};

/**
 * Simple FIFO queue with optional priority support.
 * Higher priority values are dequeued first, preserving FIFO order for equal priorities.
 */
export class CommandQueue<T> {
  private readonly entries: QueueEntry<T>[] = [];
  private readonly defaultPriority: number;

  constructor(defaultPriority = 0) {
    this.defaultPriority = defaultPriority;
  }

  enqueue(value: T, options: QueueInsertOptions = {}): void {
    const priority = options.priority ?? this.defaultPriority;
    const entry: QueueEntry<T> = {
      value,
      priority,
      enqueuedAt: Date.now(),
    };

    if (this.entries.length === 0) {
      this.entries.push(entry);
      return;
    }

    const index = this.entries.findIndex((existing) => {
      if (priority === existing.priority) {
        return entry.enqueuedAt < existing.enqueuedAt;
      }
      return priority > existing.priority;
    });

    if (index === -1) {
      this.entries.push(entry);
    } else {
      this.entries.splice(index, 0, entry);
    }
  }

  dequeue(): T | undefined {
    const entry = this.entries.shift();
    return entry?.value;
  }

  peek(): T | undefined {
    return this.entries[0]?.value;
  }

  remove(predicate: (value: T) => boolean): T | undefined {
    const index = this.entries.findIndex((entry) => predicate(entry.value));
    if (index === -1) return undefined;
    const [removed] = this.entries.splice(index, 1);
    return removed.value;
  }

  clear(): T[] {
    const values = this.entries.map((entry) => entry.value);
    this.entries.length = 0;
    return values;
  }

  get size(): number {
    return this.entries.length;
  }

  isEmpty(): boolean {
    return this.entries.length === 0;
  }

  values(): T[] {
    return this.entries.map((entry) => entry.value);
  }
}

export default CommandQueue;
