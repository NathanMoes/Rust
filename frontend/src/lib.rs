use yew::prelude::*;
use yew_router::prelude::*;

mod components;
mod pages;
mod services;
mod types;

use components::navbar::Navbar;
use pages::{Home, Artists, Tracks, Playlists, Recommendations};

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/artists")]
    Artists,
    #[at("/tracks")]
    Tracks,
    #[at("/playlists")]
    Playlists,
    #[at("/recommendations")]
    Recommendations,
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <Home /> },
        Route::Artists => html! { <Artists /> },
        Route::Tracks => html! { <Tracks /> },
        Route::Playlists => html! { <Playlists /> },
        Route::Recommendations => html! { <Recommendations /> },
    }
}

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <BrowserRouter>
            <div class="min-h-screen bg-gray-100">
                <Navbar />
                <main class="container mx-auto px-4 py-8">
                    <Switch<Route> render={switch} />
                </main>
            </div>
        </BrowserRouter>
    }
}

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
    yew::Renderer::<App>::new().render();
}
