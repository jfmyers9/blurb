use dioxus::prelude::*;

use crate::data::models::Book;

use super::book_card::BookCard;

#[derive(Props, Clone, PartialEq)]
pub struct LibraryGridProps {
    books: Vec<Book>,
    on_select_book: EventHandler<i64>,
}

const CARD_MIN_WIDTH: u32 = 180;

#[component]
pub fn LibraryGrid(props: LibraryGridProps) -> Element {
    let grid_style = format!(
        "grid-template-columns: repeat(auto-fill, minmax({}px, 1fr))",
        CARD_MIN_WIDTH
    );

    rsx! {
        div {
            class: "grid gap-4 p-6",
            style: "{grid_style}",
            for book in props.books.iter() {
                BookCard {
                    key: "{book.id}",
                    book: book.clone(),
                    on_click: move |id| props.on_select_book.call(id),
                }
            }
        }
    }
}
