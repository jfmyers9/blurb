import { convertFileSrc } from "@tauri-apps/api/core";

export function coverSrc(url: string): string {
  if (url.startsWith("/") || url.match(/^[A-Za-z]:\\/)) {
    return convertFileSrc(url);
  }
  return url;
}
