export function invoke() {
  return Promise.resolve(null);
}

export function convertFileSrc(path: string) {
  return `asset://localhost/${encodeURIComponent(path)}`;
}
