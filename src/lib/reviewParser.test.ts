import { describe, it, expect } from "vitest";
import { parsePlainTextToDoc, parseReviewContent } from "./reviewParser";

describe("parseReviewContent", () => {
  it("returns undefined for null", () => {
    expect(parseReviewContent(null)).toBeUndefined();
  });

  it("parses valid JSON doc", () => {
    const input = '{"type":"doc","content":[]}';
    expect(parseReviewContent(input)).toEqual({ type: "doc", content: [] });
  });

  it("converts plain text to doc with paragraph", () => {
    const result = parseReviewContent("plain text");
    expect(result).toEqual({
      type: "doc",
      content: [
        {
          type: "paragraph",
          content: [{ type: "text", text: "plain text" }],
        },
      ],
    });
  });
});

describe("parsePlainTextToDoc", () => {
  it("splits double newlines into separate paragraphs", () => {
    const result = parsePlainTextToDoc("para1\n\npara2");
    expect(result.content).toHaveLength(2);
    expect(result.content[0]).toEqual({
      type: "paragraph",
      content: [{ type: "text", text: "para1" }],
    });
    expect(result.content[1]).toEqual({
      type: "paragraph",
      content: [{ type: "text", text: "para2" }],
    });
  });

  it("converts single newlines to hardBreak nodes", () => {
    const result = parsePlainTextToDoc("line1\nline2");
    expect(result.content).toHaveLength(1);
    expect(result.content[0]).toEqual({
      type: "paragraph",
      content: [
        { type: "text", text: "line1" },
        { type: "hardBreak" },
        { type: "text", text: "line2" },
      ],
    });
  });
});
