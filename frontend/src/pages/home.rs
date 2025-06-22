use yew::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::services::api::ApiService;
use crate::components::common::{Alert, Button, Card};

#[function_component(Home)]
pub fn home() -> Html {
    let status = use_state(|| "Ready".to_string());
    let is_loading = use_state(|| false);
    let playlist_url = use_state(|| String::new());
    let alert_message = use_state(|| None::<(String, bool)>);

    let check_health = {
        let status = status.clone();
        let is_loading = is_loading.clone();
        let alert_message = alert_message.clone();
        
        Callback::from(move |_| {
            let status = status.clone();
            let is_loading = is_loading.clone();
            let alert_message = alert_message.clone();
            
            spawn_local(async move {
                is_loading.set(true);
                match ApiService::health_check().await {
                    Ok(response) => {
                        status.set("Connected".to_string());
                        alert_message.set(Some((response, false)));
                    }
                    Err(error) => {
                        status.set("Error".to_string());
                        alert_message.set(Some((error, true)));
                    }
                }
                is_loading.set(false);
            });
        })
    };

    let import_spotify_data = {
        let playlist_url = playlist_url.clone();
        let is_loading = is_loading.clone();
        let alert_message = alert_message.clone();
        
        Callback::from(move |_| {
            let url = (*playlist_url).clone();
            let is_loading = is_loading.clone();
            let alert_message = alert_message.clone();
            
            if url.trim().is_empty() {
                alert_message.set(Some(("Please enter a Spotify playlist URL".to_string(), true)));
                return;
            }
            
            spawn_local(async move {
                is_loading.set(true);
                match ApiService::import_spotify_data(url).await {
                    Ok(response) => {
                        alert_message.set(Some((response, false)));
                    }
                    Err(error) => {
                        alert_message.set(Some((error, true)));
                    }
                }
                is_loading.set(false);
            });
        })
    };

    let on_url_change = {
        let playlist_url = playlist_url.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            playlist_url.set(input.value());
        })
    };

    html! {
        <div class="max-w-4xl mx-auto space-y-8">
            <div class="text-center">
                <h1 class="text-4xl font-bold text-gray-900 mb-4">
                    {"🎵 Spotify Neo4j Backend"}
                </h1>
                <p class="text-xl text-gray-600 mb-8">
                    {"Create intelligent YouTube playlists from Spotify data using graph database relationships"}
                </p>
            </div>

            if let Some((message, is_error)) = (*alert_message).clone() {
                <Alert message={message} error={is_error} />
            }

            <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                <Card title="System Status">
                    <div class="space-y-4">
                        <div class="flex items-center justify-between">
                            <span class="text-gray-700">{"Backend Status:"}</span>
                            <span class={format!("px-2 py-1 rounded text-sm font-medium {}", 
                                match status.as_str() {
                                    "Connected" => "bg-green-100 text-green-800",
                                    "Error" => "bg-red-100 text-red-800",
                                    _ => "bg-gray-100 text-gray-800"
                                }
                            )}>
                                {&*status}
                            </span>
                        </div>
                        <Button
                            onclick={check_health}
                            disabled={Some(*is_loading)}
                            variant="primary"
                        >
                            if *is_loading {
                                {"Checking..."}
                            } else {
                                {"Check Connection"}
                            }
                        </Button>
                    </div>
                </Card>

                <Card title="Import Spotify Data">
                    <div class="space-y-4">
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-2">
                                {"Spotify Playlist URL"}
                            </label>
                            <input
                                type="url"
                                class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-purple-500"
                                placeholder="https://open.spotify.com/playlist/..."
                                value={(*playlist_url).clone()}
                                oninput={on_url_change}
                            />
                        </div>
                        <Button
                            onclick={import_spotify_data}
                            disabled={Some(*is_loading)}
                            variant="primary"
                        >
                            if *is_loading {
                                {"Importing..."}
                            } else {
                                {"Import Playlist"}
                            }
                        </Button>
                    </div>
                </Card>
            </div>

            <Card title="Features">
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
                    <div class="text-center p-4">
                        <div class="text-3xl mb-2">{"🎵"}</div>
                        <h3 class="font-semibold">{"Spotify Import"}</h3>
                        <p class="text-sm text-gray-600">{"Import playlist data and audio features"}</p>
                    </div>
                    <div class="text-center p-4">
                        <div class="text-3xl mb-2">{"🔗"}</div>
                        <h3 class="font-semibold">{"Graph Database"}</h3>
                        <p class="text-sm text-gray-600">{"Store relationships in Neo4j"}</p>
                    </div>
                    <div class="text-center p-4">
                        <div class="text-3xl mb-2">{"🎯"}</div>
                        <h3 class="font-semibold">{"Recommendations"}</h3>
                        <p class="text-sm text-gray-600">{"AI-powered music suggestions"}</p>
                    </div>
                    <div class="text-center p-4">
                        <div class="text-3xl mb-2">{"📺"}</div>
                        <h3 class="font-semibold">{"YouTube Playlists"}</h3>
                        <p class="text-sm text-gray-600">{"Auto-create YouTube playlists"}</p>
                    </div>
                </div>
            </Card>
        </div>
    }
}
