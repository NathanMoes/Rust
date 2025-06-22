use yew::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::services::api::ApiService;
use crate::types::{Track, CreatedPlaylist};

#[function_component(Playlists)]
pub fn playlists() -> Html {
    let tracks = use_state(|| Vec::<Track>::new());
    let selected_track_id = use_state(|| String::new());
    let playlist_title = use_state(|| String::new());
    let playlist_description = use_state(|| String::new());
    let created_playlist = use_state(|| None::<CreatedPlaylist>);
    let track_queries = use_state(|| String::new());
    let recommendation_limit = use_state(|| 10u32);
    
    let is_loading_tracks = use_state(|| false);
    let is_loading_playlist = use_state(|| false);
    let error_message = use_state(|| None::<String>);
    let active_tab = use_state(|| "manual".to_string());

    // Load tracks on component mount
    {
        let tracks = tracks.clone();
        let is_loading_tracks = is_loading_tracks.clone();
        let error_message = error_message.clone();
        
        use_effect_with((), move |_| {
            spawn_local(async move {
                is_loading_tracks.set(true);
                match ApiService::get_tracks().await {
                    Ok(data) => {
                        tracks.set(data);
                        error_message.set(None);
                    }
                    Err(error) => {
                        error_message.set(Some(error));
                    }
                }
                is_loading_tracks.set(false);
            });
            || ()
        });
    }

    let create_manual_playlist = {
        let playlist_title = playlist_title.clone();
        let playlist_description = playlist_description.clone();
        let track_queries = track_queries.clone();
        let created_playlist = created_playlist.clone();
        let is_loading_playlist = is_loading_playlist.clone();
        let error_message = error_message.clone();
        
        Callback::from(move |_| {
            let title = (*playlist_title).clone();
            let description = (*playlist_description).clone();
            let queries = (*track_queries).clone();
            let created_playlist = created_playlist.clone();
            let is_loading_playlist = is_loading_playlist.clone();
            let error_message = error_message.clone();
            
            if title.trim().is_empty() {
                error_message.set(Some("Please enter a playlist title".to_string()));
                return;
            }
            
            if queries.trim().is_empty() {
                error_message.set(Some("Please enter some track queries".to_string()));
                return;
            }
            
            let query_list: Vec<String> = queries
                .lines()
                .map(|line| line.trim().to_string())
                .filter(|line| !line.is_empty())
                .collect();
            
            if query_list.is_empty() {
                error_message.set(Some("Please enter valid track queries".to_string()));
                return;
            }
            
            spawn_local(async move {
                is_loading_playlist.set(true);
                match ApiService::create_youtube_playlist(title, description, query_list).await {
                    Ok(playlist) => {
                        created_playlist.set(Some(playlist));
                        error_message.set(None);
                    }
                    Err(error) => {
                        error_message.set(Some(error));
                    }
                }
                is_loading_playlist.set(false);
            });
        })
    };

    let create_recommendation_playlist = {
        let playlist_title = playlist_title.clone();
        let playlist_description = playlist_description.clone();
        let selected_track_id = selected_track_id.clone();
        let recommendation_limit = recommendation_limit.clone();
        let created_playlist = created_playlist.clone();
        let is_loading_playlist = is_loading_playlist.clone();
        let error_message = error_message.clone();
        
        Callback::from(move |_| {
            let title = (*playlist_title).clone();
            let description = (*playlist_description).clone();
            let track_id = (*selected_track_id).clone();
            let limit = *recommendation_limit;
            let created_playlist = created_playlist.clone();
            let is_loading_playlist = is_loading_playlist.clone();
            let error_message = error_message.clone();
            
            if title.trim().is_empty() {
                error_message.set(Some("Please enter a playlist title".to_string()));
                return;
            }
            
            if track_id.is_empty() {
                error_message.set(Some("Please select a track for recommendations".to_string()));
                return;
            }
            
            spawn_local(async move {
                is_loading_playlist.set(true);
                match ApiService::create_playlist_from_recommendations(title, description, track_id, Some(limit)).await {
                    Ok(playlist) => {
                        created_playlist.set(Some(playlist));
                        error_message.set(None);
                    }
                    Err(error) => {
                        error_message.set(Some(error));
                    }
                }
                is_loading_playlist.set(false);
            });
        })
    };

    let on_tab_change = {
        let active_tab = active_tab.clone();
        let created_playlist = created_playlist.clone();
        Callback::from(move |tab: String| {
            active_tab.set(tab);
            created_playlist.set(None);
        })
    };

    html! {
        <div class="max-w-4xl mx-auto space-y-6">
            <h1 class="text-3xl font-bold text-gray-900">{"Create YouTube Playlists"</h1>

            if let Some(error) = (*error_message).clone() {
                <Alert message={error} error={true} />
            }

            // Tab Navigation
            <div class="border-b border-gray-200">
                <nav class="-mb-px flex space-x-8">
                    <button
                        class={format!("py-2 px-1 border-b-2 font-medium text-sm {}",
                            if *active_tab == "manual" {
                                "border-purple-500 text-purple-600"
                            } else {
                                "border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300"
                            }
                        )}
                        onclick={
                            let on_tab_change = on_tab_change.clone();
                            Callback::from(move |_| on_tab_change.emit("manual".to_string()))
                        }
                    >
                        {"Manual Track List"
                    </button>
                    
                    <button
                        class={format!("py-2 px-1 border-b-2 font-medium text-sm {}",
                            if *active_tab == "recommendations" {
                                "border-purple-500 text-purple-600"
                            } else {
                                "border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300"
                            }
                        )}
                        onclick={
                            let on_tab_change = on_tab_change.clone();
                            Callback::from(move |_| on_tab_change.emit("recommendations".to_string()))
                        }
                    >
                        {"From Recommendations"
                    </button>
                </nav>
            </div>

            // Common playlist info
            <Card title="Playlist Information">
                <div class="space-y-4">
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-2">
                            {"Playlist Title *"
                        </label>
                        <input
                            type="text"
                            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-purple-500"
                            placeholder="My Awesome Playlist"
                            value={(*playlist_title).clone()}
                            oninput={
                                let playlist_title = playlist_title.clone();
                                Callback::from(move |e: InputEvent| {
                                    let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                    playlist_title.set(input.value());
                                })
                            }
                        />
                    </div>
                    
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-2">
                            {"Description"
                        </label>
                        <textarea
                            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-purple-500"
                            rows="3"
                            placeholder="Created with Spotify Neo4j Backend"
                            value={(*playlist_description).clone()}
                            oninput={
                                let playlist_description = playlist_description.clone();
                                Callback::from(move |e: InputEvent| {
                                    let textarea: web_sys::HtmlTextAreaElement = e.target_unchecked_into();
                                    playlist_description.set(textarea.value());
                                })
                            }
                        />
                    </div>
                </div>
            </Card>

            {if *active_tab == "manual" {
                html! {
                    <Card title="Manual Track Queries">
                        <div class="space-y-4">
                            <div>
                                <label class="block text-sm font-medium text-gray-700 mb-2">
                                    {"Track Queries (one per line) *"
                                </label>
                                <textarea
                                    class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-purple-500"
                                    rows="10"
                                    placeholder={concat!(
                                        "Enter track queries, one per line:\n",
                                        "The Beatles - Hey Jude\n",
                                        "Queen - Bohemian Rhapsody\n",
                                        "Led Zeppelin - Stairway to Heaven"
                                    )}
                                    value={(*track_queries).clone()}
                                    oninput={
                                        let track_queries = track_queries.clone();
                                        Callback::from(move |e: InputEvent| {
                                            let textarea: web_sys::HtmlTextAreaElement = e.target_unchecked_into();
                                            track_queries.set(textarea.value());
                                        })
                                    }
                                />
                                <p class="text-sm text-gray-600 mt-1">
                                    {"Format: Artist - Song Title (one per line)"
                                </p>
                            </div>
                            
                            <Button
                                onclick={create_manual_playlist}
                                disabled={Some(*is_loading_playlist)}
                                variant="primary"
                            >
                                if *is_loading_playlist {
                                    {"Creating Playlist..."
                                } else {
                                    {"Create YouTube Playlist"
                                }
                            </Button>
                        </div>
                    </Card>
                }
            } else {
                html! {
                    <Card title="Create from Recommendations">
                        <div class="space-y-4">
                            <div>
                                <label class="block text-sm font-medium text-gray-700 mb-2">
                                    {"Select a track to base recommendations on: *"
                                </label>
                                {if *is_loading_tracks {
                                    <div class="text-gray-500">{"Loading tracks..."</div>
                                } else if tracks.is_empty() {
                                    <div class="text-gray-500">
                                        {"No tracks available. Import a Spotify playlist first."
                                    </div>
                                } else {
                                    html! {
                                        <select
                                            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-purple-500"
                                            value={(*selected_track_id).clone()}
                                            onchange={
                                                let selected_track_id = selected_track_id.clone();
                                                Callback::from(move |e: Event| {
                                                    let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
                                                    selected_track_id.set(select.value());
                                                })
                                            }
                                        >
                                            <option value="">{"-- Select a track --"</option>
                                            {for tracks.iter().map(|track| {
                                                html! {
                                                    <option value={track.id.clone()}>
                                                        {format!("{} - {}", track.name, track.artist_names.join(", "))}
                                                    </option>
                                                }
                                            })}
                                        </select>
                                    }
                                }}
                            </div>
                            
                            <div>
                                <label class="block text-sm font-medium text-gray-700 mb-2">
                                    {"Number of recommended tracks:"
                                </label>
                                <input
                                    type="number"
                                    min="1"
                                    max="50"
                                    value={recommendation_limit.to_string()}
                                    oninput={
                                        let recommendation_limit = recommendation_limit.clone();
                                        Callback::from(move |e: InputEvent| {
                                            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                            if let Ok(value) = input.value().parse::<u32>() {
                                                recommendation_limit.set(value.max(1).min(50));
                                            }
                                        })
                                    }
                                    class="w-24 px-2 py-1 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-purple-500"
                                />
                            </div>
                            
                            <Button
                                onclick={create_recommendation_playlist}
                                disabled={Some(*is_loading_playlist || selected_track_id.is_empty())}
                                variant="primary"
                            >
                                if *is_loading_playlist {
                                    {"Creating Playlist from Recommendations..."
                                } else {
                                    {"Create Playlist from Recommendations"
                                }
                            </Button>
                        </div>
                    </Card>
                }
            }}

            {if *is_loading_playlist {
                <div class="flex items-center justify-center py-8">
                    <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-purple-600"></div>
                    <span class="ml-3 text-gray-600">{"Creating your YouTube playlist..."</span>
                </div>
            }}

            {if let Some(playlist) = (*created_playlist).clone() {
                <Card title="Playlist Created Successfully! 🎉">
                        <div class="space-y-4">
                            <div class="bg-green-50 border border-green-200 rounded-lg p-4">
                                <h4 class="font-semibold text-green-800 text-lg">{&playlist.title}</h4>
                                <p class="text-green-700 mt-1">{"Playlist ID: "{&playlist.playlist_id}</p>

                                <div class="mt-4">
                                    <a
                                        href={playlist.url.clone()}
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        class="inline-flex items-center px-4 py-2 bg-red-600 text-white rounded-md hover:bg-red-700 transition-colors"
                                    >
                                        {"🎵 Open in YouTube"
                                        <svg class="ml-2 w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"></path>
                                        </svg>
                                    </a>
                                </div>
                            </div>
                        </div>
                    </Card>
            }}
        </div>
    }
}
