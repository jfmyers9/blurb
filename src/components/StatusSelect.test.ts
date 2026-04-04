import { describe, it, expect } from "vitest";
import { getStatusInfo } from "./StatusSelect";

describe("getStatusInfo", () => {
  it("returns correct label for known status", () => {
    expect(getStatusInfo("reading").label).toBe("Reading");
  });

  it("returns No status for null", () => {
    expect(getStatusInfo(null).label).toBe("No status");
  });

  it("returns No status for unknown values", () => {
    expect(getStatusInfo("unknown").label).toBe("No status");
  });
});
