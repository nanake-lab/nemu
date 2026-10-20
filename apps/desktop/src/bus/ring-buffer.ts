export class RingBuffer<T> {
  private buffer: Array<T | undefined>;
  private head = 0;
  private tail = 0;
  private count = 0;
  public readonly capacity: number;

  constructor(capacity = 5000) {
    if (capacity <= 0) {
      throw new Error('RingBuffer capacity must be greater than 0');
    }
    this.capacity = capacity;
    this.buffer = new Array<T | undefined>(capacity);
  }

  public push(item: T): void {
    if (this.count === this.capacity) {
      // Overwrite oldest item at head
      this.buffer[this.head] = item;
      this.head = (this.head + 1) % this.capacity;
      this.tail = (this.tail + 1) % this.capacity;
    } else {
      this.buffer[this.tail] = item;
      this.tail = (this.tail + 1) % this.capacity;
      this.count++;
    }
  }

  public get length(): number {
    return this.count;
  }

  public get(index: number): T | undefined {
    if (index < 0 || index >= this.count) return undefined;
    const actualIndex = (this.head + index) % this.capacity;
    return this.buffer[actualIndex];
  }

  public getLatest(): T | undefined {
    if (this.count === 0) return undefined;
    const latestIndex = (this.tail - 1 + this.capacity) % this.capacity;
    return this.buffer[latestIndex];
  }

  public toArray(): T[] {
    const result: T[] = new Array<T>(this.count);
    for (let i = 0; i < this.count; i++) {
      result[i] = this.buffer[(this.head + i) % this.capacity] as T;
    }
    return result;
  }

  public clear(): void {
    this.buffer = new Array<T | undefined>(this.capacity);
    this.head = 0;
    this.tail = 0;
    this.count = 0;
  }
}
