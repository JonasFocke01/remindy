use leptos::*;

#[derive(serde::Deserialize, serde::Serialize, Clone)]
struct Cat {
    id: usize,
    name: String,
}

#[component]
pub fn App() -> impl IntoView {
    let (count, set_count) = create_signal(0);

    let reminders = create_resource(
        || (),
        |_| async move {
            reqwasm::http::Request::get(&format!("http://192.168.2.95:6969/reminders"))
                .send()
                .await
                .unwrap()
                .json::<Vec<Cat>>()
                .await
                .unwrap()
        },
    );

    view! {
        <button
            on:click=move |_| {
                set_count.update(|n| *n += 1)
            }
        >
            "Click me: "
            {move || count()}
        </button>
            "reminders_count: "
            {move || reminders().and_then(|data| {
                                         Some(data
                                              .iter()
                                              .map(|s| view! {
                                                  <p>{s.name.clone()}</p>
                                              })
                                              .collect_view()
                                         )
                                     }
                             )
            }
    }
}
