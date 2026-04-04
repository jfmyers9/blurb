export function parsePlainTextToDoc(text: string) {
  const paragraphs = text.split(/\n\n+/);
  return {
    type: "doc" as const,
    content: paragraphs.map((p) => ({
      type: "paragraph" as const,
      content: p.split(/\n/).flatMap((line, i, arr) => {
        const nodes: Array<{ type: string; text?: string }> = [];
        if (line) nodes.push({ type: "text", text: line });
        if (i < arr.length - 1) nodes.push({ type: "hardBreak" });
        return nodes;
      }),
    })),
  };
}

export function parseReviewContent(review: string | null) {
  if (!review) return undefined;
  try {
    const parsed = JSON.parse(review);
    if (parsed?.type === "doc") return parsed;
  } catch {
    // plain text fallback
  }
  return parsePlainTextToDoc(review);
}
