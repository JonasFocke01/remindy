use leptos::*;

mod App;
use App::App;

fn main() {
    mount_to_body(|| {
        view! {
            <App />
        }
    })
}
