import { describe, it, expect } from "vitest";
import { getStatusInfo } from "./StatusSelect";

describe("getStatusInfo", () => {
  it("returns Want to Read for want_to_read", () => {
    expect(getStatusInfo("want_to_read").label).toBe("Want to Read");
  });

  it("returns Reading for reading", () => {
    expect(getStatusInfo("reading").label).toBe("Reading");
  });

  it("returns Finished for finished", () => {
    expect(getStatusInfo("finished").label).toBe("Finished");
  });

  it("returns Abandoned for abandoned", () => {
    expect(getStatusInfo("abandoned").label).toBe("Abandoned");
  });

  it("returns No status for null", () => {
    expect(getStatusInfo(null).label).toBe("No status");
  });

  it("returns No status for unknown values", () => {
    expect(getStatusInfo("unknown").label).toBe("No status");
  });
});
