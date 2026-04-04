export function invoke<T>(cmd: string, _args?: Record<string, unknown>): Promise<T> {
  return Promise.reject(new Error(`unmocked invoke: ${cmd}`));
}

export function convertFileSrc(path: string): string {
  return `asset://mock/${path}`;
}
