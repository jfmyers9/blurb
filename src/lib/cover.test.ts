import { describe, it, expect } from "vitest";
import { coverSrc } from "./cover";

describe("coverSrc", () => {
  it("returns HTTP URLs as-is", () => {
    expect(coverSrc("https://example.com/cover.jpg")).toBe("https://example.com/cover.jpg");
  });

  it("converts absolute Unix paths via convertFileSrc", () => {
    expect(coverSrc("/path/to/cover.jpg")).toMatch(/^asset:\/\//);
  });

  it("converts Windows paths via convertFileSrc", () => {
    expect(coverSrc("C:\\path\\cover.jpg")).toMatch(/^asset:\/\//);
  });
});
