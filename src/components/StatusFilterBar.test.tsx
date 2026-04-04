import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import StatusFilterBar from "./StatusFilterBar";
import type { FilterStatus } from "./StatusFilterBar";
import type { Book, Shelf } from "../lib/api";

function makeBook(overrides: Partial<Book> = {}): Book {
  return {
    id: 1,
    title: "Test Book",
    author: "Author",
    isbn: null,
    asin: null,
    cover_url: null,
    description: null,
    publisher: null,
    published_date: null,
    page_count: null,
    created_at: "2024-01-01",
    updated_at: "2024-01-01",
    rating: null,
    status: null,
    started_at: null,
    finished_at: null,
    ...overrides,
  };
}

const defaultProps = () => ({
  books: [] as Book[],
  activeStatus: "all" as FilterStatus,
  onStatusChange: vi.fn<(status: FilterStatus) => void>(),
  sortBy: "date_added" as const,
  onSortChange: vi.fn(),
  shelves: [] as Shelf[],
  activeShelf: null,
  onShelfChange: vi.fn(),
  shelfBookCounts: {} as Record<number, number>,
  onRenameShelf: vi.fn(),
  onDeleteShelf: vi.fn(),
  searchQuery: "",
  onSearchChange: vi.fn(),
  viewMode: "grid" as const,
  onViewModeChange: vi.fn(),
  minRating: null,
  onMinRatingChange: vi.fn(),
  searchInputRef: { current: null },
  onClearAll: vi.fn(),
});

describe("StatusFilterBar", () => {
  it("renders all five status tabs", () => {
    render(<StatusFilterBar {...defaultProps()} />);
    expect(screen.getByText("All")).toBeInTheDocument();
    expect(screen.getByText("Want to Read")).toBeInTheDocument();
    expect(screen.getByText("Reading")).toBeInTheDocument();
    expect(screen.getByText("Finished")).toBeInTheDocument();
    expect(screen.getByText("Abandoned")).toBeInTheDocument();
  });

  it("shows correct book counts per status", () => {
    const books = [
      makeBook({ id: 1, status: "reading" }),
      makeBook({ id: 2, status: "reading" }),
      makeBook({ id: 3, status: "finished" }),
    ];
    render(<StatusFilterBar {...defaultProps()} books={books} />);

    // "All" tab shows total count
    const allBtn = screen.getByText("All").closest("button")!;
    expect(allBtn).toHaveTextContent("3");

    const readingBtn = screen.getByText("Reading").closest("button")!;
    expect(readingBtn).toHaveTextContent("2");

    const finishedBtn = screen.getByText("Finished").closest("button")!;
    expect(finishedBtn).toHaveTextContent("1");

    // Tabs with zero books still show 0
    const wantBtn = screen.getByText("Want to Read").closest("button")!;
    expect(wantBtn).toHaveTextContent("0");
  });

  it("calls onStatusChange with the clicked tab value", () => {
    const props = defaultProps();
    render(<StatusFilterBar {...props} />);

    fireEvent.click(screen.getByText("Reading"));
    expect(props.onStatusChange).toHaveBeenCalledWith("reading");

    fireEvent.click(screen.getByText("All"));
    expect(props.onStatusChange).toHaveBeenCalledWith("all");
  });

  it("renders all sort options in the dropdown", () => {
    render(<StatusFilterBar {...defaultProps()} />);
    const trigger = screen.getByLabelText("Sort by");
    fireEvent.click(trigger);
    const listbox = screen.getByRole("listbox");
    const options = Array.from(listbox.querySelectorAll('[role="option"]'));
    const labels = options.map((o) => o.textContent);
    expect(labels).toEqual(["Date Added", "Title", "Author", "Rating"]);
  });

  it("calls onSortChange when sort dropdown option is clicked", () => {
    const props = defaultProps();
    render(<StatusFilterBar {...props} />);
    fireEvent.click(screen.getByLabelText("Sort by"));
    fireEvent.mouseDown(screen.getByText("Title"));
    expect(props.onSortChange).toHaveBeenCalledWith("title");
  });

  it("renders shelf pills with names and counts", () => {
    const shelves: Shelf[] = [
      { id: 1, name: "Fiction", created_at: "2024-01-01" },
      { id: 2, name: "Sci-Fi", created_at: "2024-01-01" },
    ];
    render(
      <StatusFilterBar
        {...defaultProps()}
        shelves={shelves}
        shelfBookCounts={{ 1: 5, 2: 3 }}
      />
    );

    expect(screen.getByText("Fiction")).toBeInTheDocument();
    expect(screen.getByText("Sci-Fi")).toBeInTheDocument();
    expect(screen.getByText("All Shelves")).toBeInTheDocument();
    // Counts rendered inside the pill buttons
    expect(screen.getByText("5")).toBeInTheDocument();
    expect(screen.getByText("3")).toBeInTheDocument();
  });

  it("does not render shelf row when no shelves exist", () => {
    render(<StatusFilterBar {...defaultProps()} shelves={[]} />);
    expect(screen.queryByText("All Shelves")).not.toBeInTheDocument();
  });

  it("calls onShelfChange with shelf id when clicking a shelf pill", () => {
    const props = defaultProps();
    const shelves: Shelf[] = [
      { id: 7, name: "History", created_at: "2024-01-01" },
    ];
    render(
      <StatusFilterBar {...props} shelves={shelves} shelfBookCounts={{ 7: 2 }} />
    );

    fireEvent.click(screen.getByText("History"));
    expect(props.onShelfChange).toHaveBeenCalledWith(7);
  });

  it("commits rename on blur", async () => {
    const props = defaultProps();
    const shelves: Shelf[] = [{ id: 1, name: "Old", created_at: "2024-01-01" }];
    props.onRenameShelf = vi.fn().mockResolvedValue(undefined);
    render(<StatusFilterBar {...props} shelves={shelves} shelfBookCounts={{ 1: 0 }} />);

    const renameBtn = screen.getByTitle("Rename shelf");
    fireEvent.click(renameBtn);

    const input = screen.getByDisplayValue("Old");
    fireEvent.change(input, { target: { value: "New" } });
    fireEvent.blur(input);

    await vi.waitFor(() => {
      expect(props.onRenameShelf).toHaveBeenCalledWith(1, "New");
    });
  });

  it("cancels rename on Escape without calling onRenameShelf", () => {
    const props = defaultProps();
    const shelves: Shelf[] = [{ id: 1, name: "Keep", created_at: "2024-01-01" }];
    render(<StatusFilterBar {...props} shelves={shelves} shelfBookCounts={{ 1: 0 }} />);

    fireEvent.click(screen.getByTitle("Rename shelf"));
    const input = screen.getByDisplayValue("Keep");
    fireEvent.change(input, { target: { value: "Changed" } });
    fireEvent.keyDown(input, { key: "Escape" });

    expect(props.onRenameShelf).not.toHaveBeenCalled();
    expect(screen.getByText("Keep")).toBeInTheDocument();
  });

  it("calls onShelfChange(null) when clicking All Shelves", () => {
    const props = defaultProps();
    const shelves: Shelf[] = [
      { id: 1, name: "Fiction", created_at: "2024-01-01" },
    ];
    render(
      <StatusFilterBar {...props} shelves={shelves} shelfBookCounts={{ 1: 0 }} />
    );

    fireEvent.click(screen.getByText("All Shelves"));
    expect(props.onShelfChange).toHaveBeenCalledWith(null);
  });

  it("search input renders and calls onSearchChange when user types", () => {
    const props = defaultProps();
    render(<StatusFilterBar {...props} />);
    const input = screen.getByPlaceholderText("Search...");
    fireEvent.change(input, { target: { value: "hello" } });
    expect(props.onSearchChange).toHaveBeenCalledWith("hello");
  });

  it("renders rating pills when at least one book has a rating", () => {
    const props = defaultProps();
    props.books = [makeBook({ id: 1, rating: 4 })];
    render(<StatusFilterBar {...props} />);
    expect(screen.getByText("Any Rating")).toBeInTheDocument();
    expect(screen.getByText(/3\+/)).toBeInTheDocument();
    expect(screen.getByText(/4\+/)).toBeInTheDocument();
  });

  it("does not render rating pills when no books have ratings", () => {
    const props = defaultProps();
    props.books = [makeBook({ id: 1, rating: null })];
    render(<StatusFilterBar {...props} />);
    expect(screen.queryByText("Any Rating")).not.toBeInTheDocument();
  });

  it("clicking 3+ pill calls onMinRatingChange(3)", () => {
    const props = defaultProps();
    props.books = [makeBook({ id: 1, rating: 4 })];
    render(<StatusFilterBar {...props} />);
    fireEvent.click(screen.getByText(/3\+/));
    expect(props.onMinRatingChange).toHaveBeenCalledWith(3);
  });

  it("clicking Any Rating pill calls onMinRatingChange(null)", () => {
    const props = defaultProps();
    props.books = [makeBook({ id: 1, rating: 4 })];
    render(<StatusFilterBar {...props} />);
    fireEvent.click(screen.getByText("Any Rating"));
    expect(props.onMinRatingChange).toHaveBeenCalledWith(null);
  });

  it("grid view toggle button calls onViewModeChange('grid')", () => {
    const props = defaultProps();
    render(<StatusFilterBar {...props} />);
    fireEvent.click(screen.getByTitle("Grid view"));
    expect(props.onViewModeChange).toHaveBeenCalledWith("grid");
  });

  it("list view toggle button calls onViewModeChange('list')", () => {
    const props = defaultProps();
    render(<StatusFilterBar {...props} />);
    fireEvent.click(screen.getByTitle("List view"));
    expect(props.onViewModeChange).toHaveBeenCalledWith("list");
  });

  describe("filter summary strip", () => {
    it("renders when any filter is non-default", () => {
      const props = defaultProps();
      render(<StatusFilterBar {...props} activeStatus="reading" />);
      const summary = screen.getByTestId("filter-summary");
      expect(summary).toBeInTheDocument();
      expect(summary).toHaveTextContent("Reading");
    });

    it("hidden when all filters at defaults", () => {
      render(<StatusFilterBar {...defaultProps()} />);
      expect(screen.queryByTestId("filter-summary")).not.toBeInTheDocument();
    });

    it("shows rating tag when minRating is set", () => {
      const props = defaultProps();
      props.books = [makeBook({ id: 1, rating: 5 })];
      render(<StatusFilterBar {...props} minRating={3} />);
      expect(screen.getByTestId("filter-summary")).toBeInTheDocument();
      expect(screen.getByText(/3\+ ★/)).toBeInTheDocument();
    });

    it("shows shelf tag when activeShelf is set", () => {
      const shelves: Shelf[] = [{ id: 1, name: "Fiction", created_at: "2024-01-01" }];
      render(
        <StatusFilterBar
          {...defaultProps()}
          shelves={shelves}
          activeShelf={1}
          shelfBookCounts={{ 1: 3 }}
        />
      );
      const summary = screen.getByTestId("filter-summary");
      expect(summary).toHaveTextContent("Fiction");
    });

    it("shows search tag when searchQuery is set", () => {
      render(<StatusFilterBar {...defaultProps()} searchQuery="hello" />);
      expect(screen.getByTestId("filter-summary")).toHaveTextContent("search: hello");
    });

    it("dismiss status resets to all", () => {
      const props = defaultProps();
      render(<StatusFilterBar {...props} activeStatus="finished" />);
      fireEvent.click(screen.getByTestId("dismiss-status"));
      expect(props.onStatusChange).toHaveBeenCalledWith("all");
    });

    it("dismiss rating resets to null", () => {
      const props = defaultProps();
      props.books = [makeBook({ id: 1, rating: 5 })];
      render(<StatusFilterBar {...props} minRating={4} />);
      fireEvent.click(screen.getByTestId("dismiss-rating"));
      expect(props.onMinRatingChange).toHaveBeenCalledWith(null);
    });

    it("dismiss shelf resets to null", () => {
      const props = defaultProps();
      const shelves: Shelf[] = [{ id: 2, name: "Sci-Fi", created_at: "2024-01-01" }];
      render(
        <StatusFilterBar {...props} shelves={shelves} activeShelf={2} shelfBookCounts={{ 2: 1 }} />
      );
      fireEvent.click(screen.getByTestId("dismiss-shelf"));
      expect(props.onShelfChange).toHaveBeenCalledWith(null);
    });

    it("dismiss search resets to empty", () => {
      const props = defaultProps();
      render(<StatusFilterBar {...props} searchQuery="test" />);
      fireEvent.click(screen.getByTestId("dismiss-search"));
      expect(props.onSearchChange).toHaveBeenCalledWith("");
    });

    it("clear all calls onClearAll", () => {
      const props = defaultProps();
      render(<StatusFilterBar {...props} activeStatus="reading" />);
      fireEvent.click(screen.getByText("Clear all"));
      expect(props.onClearAll).toHaveBeenCalled();
    });
  });
});
