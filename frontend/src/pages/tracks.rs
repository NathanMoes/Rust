use yew::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::services::api::ApiService;
use crate::types::Track;
use crate::components::common::{Alert, Loading, Card};

#[function_component(Tracks)]
pub fn tracks() -> Html {
    let tracks = use_state(|| Vec::<Track>::new());
    let is_loading = use_state(|| false);
    let error_message = use_state(|| None::<String>);

    // Load tracks on component mount
    {
        let tracks = tracks.clone();
        let is_loading = is_loading.clone();
        let error_message = error_message.clone();
        
        use_effect_with((), move |_| {
            spawn_local(async move {
                is_loading.set(true);
                match ApiService::get_tracks().await {
                    Ok(data) => {
                        tracks.set(data);
                        error_message.set(None);
                    }
                    Err(error) => {
                        error_message.set(Some(error));
                    }
                }
                is_loading.set(false);
            });
            || ()
        });
    }

    let refresh_tracks = {
        let tracks = tracks.clone();
        let is_loading = is_loading.clone();
        let error_message = error_message.clone();
        
        Callback::from(move |_| {
            let tracks = tracks.clone();
            let is_loading = is_loading.clone();
            let error_message = error_message.clone();
            
            spawn_local(async move {
                is_loading.set(true);
                match ApiService::get_tracks().await {
                    Ok(data) => {
                        tracks.set(data);
                        error_message.set(None);
                    }
                    Err(error) => {
                        error_message.set(Some(error));
                    }
                }
                is_loading.set(false);
            });
        })
    };

    fn format_duration(ms: u32) -> String {
        let seconds = ms / 1000;
        let minutes = seconds / 60;
        let remaining_seconds = seconds % 60;
        format!("{}:{:02}", minutes, remaining_seconds)
    }

    fn get_audio_feature_color(value: f32) -> &'static str {
        if value >= 0.7 { "bg-green-100 text-green-800" }
        else if value >= 0.4 { "bg-yellow-100 text-yellow-800" }
        else { "bg-red-100 text-red-800" }
    }

    html! {
        <div class="max-w-7xl mx-auto">
            <div class="flex justify-between items-center mb-6">
                <h1 class="text-3xl font-bold text-gray-900">{"Tracks"}</h1>
                <button
                    class="px-4 py-2 bg-purple-600 text-white rounded-md hover:bg-purple-700 disabled:bg-purple-300"
                    onclick={refresh_tracks}
                    disabled={*is_loading}
                >
                    if *is_loading {
                        {"Refreshing..."}
                    } else {
                        {"Refresh"}
                    }
                </button>
            </div>

            if let Some(error) = (*error_message).clone() {
                <Alert message={error} error={true} />
            }

            if *is_loading && tracks.is_empty() {
                <Loading message={Some("Loading tracks...".to_string())} />
            } else if tracks.is_empty() {
                <Card title="No Tracks Found">
                    <p class="text-gray-600">
                        {"No tracks have been imported yet. Import a Spotify playlist to see tracks here."}
                    </p>
                </Card>
            } else {
                <div class="space-y-4">
                    {for tracks.iter().map(|track| {
                        html! {
                            <div class="bg-white rounded-lg shadow-md p-6 hover:shadow-lg transition-shadow">
                                <div class="flex items-start justify-between">
                                    <div class="flex-1 min-w-0">
                                        <div class="flex items-start space-x-4">
                                            <div class="flex-1">
                                                <h3 class="text-lg font-semibold text-gray-900 truncate">
                                                    {&track.name}
                                                </h3>
                                                
                                                <p class="text-gray-600 mt-1">
                                                    {"by "}{track.artist_names.join(", ")}
                                                </p>
                                                
                                                <p class="text-sm text-gray-500 mt-1">
                                                    {"Album: "}{&track.album_name}
                                                </p>
                                                
                                                <div class="flex items-center space-x-4 mt-2 text-sm text-gray-600">
                                                    <span>{format_duration(track.duration_ms)}</span>
                                                    <span>{"Popularity: "}{track.popularity}{"/100"}</span>
                                                    {if track.explicit {
                                                        html! {
                                                            <span class="px-2 py-1 bg-red-100 text-red-800 text-xs rounded">
                                                                {"EXPLICIT"}
                                                            </span>
                                                        }
                                                    } else {
                                                        html! {}
                                                    }}
                                                </div>
                                            </div>
                                            
                                            {if let Some(preview_url) = &track.preview_url {
                                                if !preview_url.is_empty() {
                                                    html! {
                                                        <audio controls={true} class="w-64">
                                                            <source src={preview_url.clone()} type="audio/mpeg" />
                                                            {"Your browser does not support the audio element."}
                                                        </audio>
                                                    }
                                                } else {
                                                    html! {}
                                                }
                                            } else {
                                                html! {}
                                            }}
                                        </div>
                                        
                                        // Audio Features
                                        <div class="mt-4 border-t pt-4">
                                            <h4 class="text-sm font-medium text-gray-700 mb-2">{"Audio Features"}</h4>
                                            <div class="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-2 text-xs">
                                                <div class={format!("px-2 py-1 rounded text-center {}", get_audio_feature_color(track.danceability))}>
                                                    <div class="font-medium">{"Dance"}</div>
                                                    <div>{format!("{:.0}%", track.danceability * 100.0)}</div>
                                                </div>
                                                
                                                <div class={format!("px-2 py-1 rounded text-center {}", get_audio_feature_color(track.energy))}>
                                                    <div class="font-medium">{"Energy"}</div>
                                                    <div>{format!("{:.0}%", track.energy * 100.0)}</div>
                                                </div>
                                                
                                                <div class={format!("px-2 py-1 rounded text-center {}", get_audio_feature_color(track.valence))}>
                                                    <div class="font-medium">{"Valence"}</div>
                                                    <div>{format!("{:.0}%", track.valence * 100.0)}</div>
                                                </div>
                                                
                                                <div class="px-2 py-1 rounded text-center bg-blue-100 text-blue-800">
                                                    <div class="font-medium">{"Tempo"}</div>
                                                    <div>{format!("{:.0}", track.tempo)}</div>
                                                </div>
                                                
                                                <div class={format!("px-2 py-1 rounded text-center {}", get_audio_feature_color(track.acousticness))}>
                                                    <div class="font-medium">{"Acoustic"}</div>
                                                    <div>{format!("{:.0}%", track.acousticness * 100.0)}</div>
                                                </div>
                                                
                                                <div class={format!("px-2 py-1 rounded text-center {}", get_audio_feature_color(track.instrumentalness))}>
                                                    <div class="font-medium">{"Instrum."}</div>
                                                    <div>{format!("{:.0}%", track.instrumentalness * 100.0)}</div>
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        }
                    })}
                </div>
            }
        </div>
    }
}
