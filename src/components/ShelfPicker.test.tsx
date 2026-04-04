import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import ShelfPicker from "./ShelfPicker";
import type { Shelf } from "../lib/api";

const shelves: Shelf[] = [
  { id: 1, name: "Fiction", created_at: "2024-01-01" },
  { id: 2, name: "Sci-Fi", created_at: "2024-01-01" },
  { id: 3, name: "History", created_at: "2024-01-01" },
];

const defaultProps = () => ({
  shelves,
  bookShelfIds: [] as number[],
  onAdd: vi.fn(),
  onRemove: vi.fn(),
  onCreate: vi.fn().mockResolvedValue({ id: 99, name: "New", created_at: "2024-01-01" }),
});

describe("ShelfPicker", () => {
  it("renders chips for assigned shelves", () => {
    render(<ShelfPicker {...defaultProps()} bookShelfIds={[1, 3]} />);
    expect(screen.getByText("Fiction")).toBeInTheDocument();
    expect(screen.getByText("History")).toBeInTheDocument();
    expect(screen.queryByText("Sci-Fi")).not.toBeInTheDocument();
  });

  it("calls onRemove when clicking the remove button on a chip", () => {
    const props = defaultProps();
    render(<ShelfPicker {...props} bookShelfIds={[2]} />);

    // The remove button is inside the chip next to the shelf name
    const chip = screen.getByText("Sci-Fi").closest("span")!;
    const removeBtn = chip.querySelector("button")!;
    fireEvent.click(removeBtn);
    expect(props.onRemove).toHaveBeenCalledWith(2);
  });

  it("shows dropdown with available shelves on focus", () => {
    render(<ShelfPicker {...defaultProps()} bookShelfIds={[1]} />);

    // Before focus, no dropdown items
    expect(screen.queryByText("Sci-Fi")).not.toBeInTheDocument();

    const input = screen.getByPlaceholderText("Add to shelf...");
    fireEvent.focus(input);

    // Dropdown shows unassigned shelves (Sci-Fi and History, not Fiction)
    expect(screen.getByText("Sci-Fi")).toBeInTheDocument();
    expect(screen.getByText("History")).toBeInTheDocument();
  });

  it("filters suggestions as user types", () => {
    render(<ShelfPicker {...defaultProps()} bookShelfIds={[]} />);
    const input = screen.getByPlaceholderText("Add to shelf...");

    fireEvent.focus(input);
    fireEvent.change(input, { target: { value: "sci" } });

    expect(screen.getByText("Sci-Fi")).toBeInTheDocument();
    expect(screen.queryByText("Fiction")).not.toBeInTheDocument();
    expect(screen.queryByText("History")).not.toBeInTheDocument();
  });

  it("calls onAdd when selecting a shelf from the dropdown", () => {
    const props = defaultProps();
    render(<ShelfPicker {...props} bookShelfIds={[]} />);

    const input = screen.getByPlaceholderText("Add to shelf...");
    fireEvent.focus(input);
    fireEvent.click(screen.getByText("Fiction"));

    expect(props.onAdd).toHaveBeenCalledWith(1);
  });

  it("shows create option when input does not match any shelf", () => {
    render(<ShelfPicker {...defaultProps()} bookShelfIds={[]} />);
    const input = screen.getByPlaceholderText("Add to shelf...");

    fireEvent.focus(input);
    fireEvent.change(input, { target: { value: "Fantasy" } });

    expect(screen.getByText('Create "Fantasy"')).toBeInTheDocument();
  });

  it("does not show create option when input exactly matches an existing shelf", () => {
    render(<ShelfPicker {...defaultProps()} bookShelfIds={[]} />);
    const input = screen.getByPlaceholderText("Add to shelf...");

    fireEvent.focus(input);
    fireEvent.change(input, { target: { value: "Fiction" } });

    expect(screen.queryByText(/Create/)).not.toBeInTheDocument();
  });

  it("calls onCreate and onAdd when clicking the create button", async () => {
    const props = defaultProps();
    render(<ShelfPicker {...props} bookShelfIds={[]} />);
    const input = screen.getByPlaceholderText("Add to shelf...");

    fireEvent.focus(input);
    fireEvent.change(input, { target: { value: "Fantasy" } });
    fireEvent.click(screen.getByText('Create "Fantasy"'));

    expect(props.onCreate).toHaveBeenCalledWith("Fantasy");
    // After the promise resolves, onAdd is called with the new shelf id
    await vi.waitFor(() => {
      expect(props.onAdd).toHaveBeenCalledWith(99);
    });
  });
});
