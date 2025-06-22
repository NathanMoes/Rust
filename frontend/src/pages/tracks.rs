use yew::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::services::api::ApiService;
use crate::types::{Track, SimilarTracksResponse};
use crate::components::common::{Alert, Loading, Card};

#[function_component(Tracks)]
pub fn tracks() -> Html {
    let tracks = use_state(|| Vec::<Track>::new());
    let is_loading = use_state(|| false);
    let error_message = use_state(|| None::<String>);
    let similar_tracks_data = use_state(|| None::<SimilarTracksResponse>);
    let loading_similar_for = use_state(|| None::<String>);

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

    let find_similar_tracks = {
        let similar_tracks_data = similar_tracks_data.clone();
        let loading_similar_for = loading_similar_for.clone();
        let error_message = error_message.clone();
        
        Callback::from(move |track_id: String| {
            let similar_tracks_data = similar_tracks_data.clone();
            let loading_similar_for = loading_similar_for.clone();
            let error_message = error_message.clone();
            let track_id_clone = track_id.clone();
            
            spawn_local(async move {
                loading_similar_for.set(Some(track_id_clone.clone()));
                match ApiService::get_similar_tracks_with_youtube(track_id_clone, Some(5)).await {
                    Ok(data) => {
                        similar_tracks_data.set(Some(data));
                        error_message.set(None);
                    }
                    Err(error) => {
                        error_message.set(Some(error));
                    }
                }
                loading_similar_for.set(None);
            });
        })
    };

    let close_similar_tracks = {
        let similar_tracks_data = similar_tracks_data.clone();
        Callback::from(move |_| {
            similar_tracks_data.set(None);
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
                                        
                                        // Find Similar Button
                                        <div class="mt-4 border-t pt-4">
                                            <button
                                                class="px-4 py-2 bg-purple-600 text-white rounded-md hover:bg-purple-700 disabled:bg-purple-300 disabled:cursor-not-allowed"
                                                onclick={
                                                    let track_id = track.id.clone();
                                                    let find_similar_tracks = find_similar_tracks.clone();
                                                    move |_| find_similar_tracks.emit(track_id.clone())
                                                }
                                                disabled={loading_similar_for.as_ref() == Some(&track.id)}
                                            >
                                                {if loading_similar_for.as_ref() == Some(&track.id) {
                                                    "Finding Similar Tracks..."
                                                } else {
                                                    "Find Similar Tracks"
                                                }}
                                            </button>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        }
                    })}
                </div>
            }

            // Similar Tracks Modal/Section
            {if let Some(similar_data) = &*similar_tracks_data {
                html! {
                    <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
                        <div class="bg-white rounded-lg max-w-4xl max-h-[80vh] overflow-y-auto p-6 m-4 w-full">
                            <div class="flex justify-between items-center mb-4">
                                <h2 class="text-2xl font-bold text-gray-900">
                                    {"Similar to: "}{&similar_data.original_track.name}
                                </h2>
                                <button
                                    class="text-gray-500 hover:text-gray-700"
                                    onclick={close_similar_tracks}
                                >
                                    <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
                                    </svg>
                                </button>
                            </div>

                            <div class="mb-4 p-4 bg-gray-50 rounded-lg">
                                <h3 class="font-semibold text-gray-800">{"Original Track"}</h3>
                                <p class="text-gray-600">
                                    {&similar_data.original_track.name}{" by "}{similar_data.original_track.artist_names.join(", ")}
                                </p>
                            </div>

                            <div class="space-y-3">
                                <h3 class="font-semibold text-gray-800 text-lg">{"Similar Tracks"}</h3>
                                {for similar_data.similar_tracks.iter().enumerate().map(|(index, track_with_youtube)| {
                                    let track = &track_with_youtube.track;
                                    html! {
                                        <div class="flex items-center space-x-4 p-4 bg-gray-50 rounded-lg">
                                            <div class="flex-shrink-0 w-8 h-8 bg-purple-100 text-purple-800 rounded-full flex items-center justify-center text-sm font-medium">
                                                {index + 1}
                                            </div>

                                            <div class="flex-1 min-w-0">
                                                <h4 class="font-medium text-gray-900 truncate">{&track.name}</h4>
                                                <p class="text-sm text-gray-600 truncate">
                                                    {"by "}{track.artist_names.join(", ")}
                                                </p>
                                                <p class="text-xs text-gray-500">{&track.album_name}</p>
                                                
                                                <div class="flex space-x-1 text-xs mt-2">
                                                    <span class="bg-blue-100 text-blue-800 px-2 py-1 rounded">
                                                        {"E: "}{format!("{:.0}%", track.energy * 100.0)}
                                                    </span>
                                                    <span class="bg-green-100 text-green-800 px-2 py-1 rounded">
                                                        {"D: "}{format!("{:.0}%", track.danceability * 100.0)}
                                                    </span>
                                                    <span class="bg-purple-100 text-purple-800 px-2 py-1 rounded">
                                                        {"V: "}{format!("{:.0}%", track.valence * 100.0)}
                                                    </span>
                                                </div>
                                            </div>

                                            // YouTube section
                                            <div class="flex-shrink-0">
                                                {if let Some(youtube_video) = &track_with_youtube.youtube_video {
                                                    html! {
                                                        <div class="text-center">
                                                            <p class="text-xs text-gray-500 mb-1">{"YouTube"}</p>
                                                            <a
                                                                href={format!("https://www.youtube.com/watch?v={}", youtube_video.id)}
                                                                target="_blank"
                                                                class="inline-flex items-center px-3 py-1 bg-red-600 text-white text-xs rounded hover:bg-red-700"
                                                            >
                                                                <svg class="w-3 h-3 mr-1" fill="currentColor" viewBox="0 0 24 24">
                                                                    <path d="M23.498 6.186a3.016 3.016 0 0 0-2.122-2.136C19.505 3.545 12 3.545 12 3.545s-7.505 0-9.377.505A3.017 3.017 0 0 0 .502 6.186C0 8.07 0 12 0 12s0 3.93.502 5.814a3.016 3.016 0 0 0 2.122 2.136c1.871.505 9.376.505 9.376.505s7.505 0 9.377-.505a3.015 3.015 0 0 0 2.122-2.136C24 15.93 24 12 24 12s0-3.93-.502-5.814zM9.545 15.568V8.432L15.818 12l-6.273 3.568z"/>
                                                                </svg>
                                                                {"Play"}
                                                            </a>
                                                            <p class="text-xs text-gray-400 mt-1 max-w-24 truncate">
                                                                {&youtube_video.title}
                                                            </p>
                                                        </div>
                                                    }
                                                } else {
                                                    html! {
                                                        <div class="text-center">
                                                            <p class="text-xs text-gray-400">{"Not found"}</p>
                                                            <p class="text-xs text-gray-400">{"on YouTube"}</p>
                                                        </div>
                                                    }
                                                }}
                                            </div>

                                            {if let Some(preview_url) = &track.preview_url {
                                                if !preview_url.is_empty() {
                                                    html! {
                                                        <audio controls={true} class="w-48">
                                                            <source src={preview_url.clone()} type="audio/mpeg" />
                                                        </audio>
                                                    }
                                                } else {
                                                    html! {}
                                                }
                                            } else {
                                                html! {}
                                            }}
                                        </div>
                                    }
                                })}
                            </div>
                        </div>
                    </div>
                }
            } else {
                html! {}
            }}
        </div>
    }
}
