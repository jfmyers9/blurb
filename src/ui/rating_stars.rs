use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct RatingStarsProps {
    rating: Option<i32>,
    on_rate: EventHandler<i32>,
    #[props(default = false)]
    small: bool,
}

#[component]
pub fn RatingStars(props: RatingStarsProps) -> Element {
    let mut hovered = use_signal(|| None::<i32>);
    let display = hovered.read().unwrap_or_else(|| props.rating.unwrap_or(0));
    let star_size = if props.small { "w-4 h-4" } else { "w-6 h-6" };

    rsx! {
        div {
            class: "flex gap-0.5",
            onmouseleave: move |_| hovered.set(None),
            for star in 1..=5 {
                button {
                    r#type: "button",
                    class: "focus:outline-none",
                    onmouseenter: move |_| hovered.set(Some(star)),
                    onclick: {
                        let on_rate = props.on_rate;
                        move |_| on_rate.call(star)
                    },
                    svg {
                        class: "{star_size} transition-colors {star_fill_class(star, display)}",
                        view_box: "0 0 20 20",
                        fill: "currentColor",
                        path {
                            d: "M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z",
                        }
                    }
                }
            }
        }
    }
}

fn star_fill_class(star: i32, display: i32) -> &'static str {
    if star <= display {
        "text-amber-400 fill-amber-400"
    } else {
        "text-gray-300 dark:text-gray-600 fill-current"
    }
}
