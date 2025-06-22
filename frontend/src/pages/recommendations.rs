use yew::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::services::api::ApiService;
use crate::types::Track;
use crate::components::common::{/*Alert,*/ Loading, Card, Button};

#[function_component(Recommendations)]
pub fn recommendations() -> Html {
    let tracks = use_state(|| Vec::<Track>::new());
    let recommendations = use_state(|| Vec::<Track>::new());
    let selected_track_id = use_state(|| String::new());
    let is_loading_tracks = use_state(|| false);
    let is_loading_recommendations = use_state(|| false);
    let error_message = use_state(|| None::<String>);
    let limit = use_state(|| 10u32);

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

    let get_recommendations = {
        let selected_track_id = selected_track_id.clone();
        let recommendations = recommendations.clone();
        let is_loading_recommendations = is_loading_recommendations.clone();
        let error_message = error_message.clone();
        let limit = limit.clone();
        
        Callback::from(move |_| {
            let track_id = (*selected_track_id).clone();
            let recommendations = recommendations.clone();
            let is_loading_recommendations = is_loading_recommendations.clone();
            let error_message = error_message.clone();
            let limit = *limit;
            
            if track_id.is_empty() {
                error_message.set(Some("Please select a track first".to_string()));
                return;
            }
            
            spawn_local(async move {
                is_loading_recommendations.set(true);
                match ApiService::get_recommendations(track_id, Some(limit)).await {
                    Ok(data) => {
                        recommendations.set(data);
                        error_message.set(None);
                    }
                    Err(error) => {
                        error_message.set(Some(error));
                    }
                }
                is_loading_recommendations.set(false);
            });
        })
    };

    let on_track_select = {
        let selected_track_id = selected_track_id.clone();
        Callback::from(move |e: Event| {
            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
            selected_track_id.set(select.value());
        })
    };

    let on_limit_change = {
        let limit = limit.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            if let Ok(value) = input.value().parse::<u32>() {
                limit.set(value.max(1).min(50));
            }
        })
    };

    let selected_track = tracks.iter().find(|t| t.id == *selected_track_id);

    html! {
        <div class="max-w-6xl mx-auto space-y-6">
            <h1 class="text-3xl font-bold text-gray-900">{"Music Recommendations"}</h1>

            //{if let Some(error) = (*error_message).clone() {
            //    <Alert message={error} error={true} />
            //}}

            <Card title="Find Similar Tracks">
                <div class="space-y-4">
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-2">
                            {"Select a track to get recommendations for:"}
                        </label>
                        {
                            if *is_loading_tracks {
                                html! { <div class="text-gray-500">{"Loading tracks..."}</div> }
                            } else if tracks.is_empty() {
                                html! {
                                    <div class="text-gray-500">
                                        {"No tracks available. Import a Spotify playlist first."}
                                    </div>
                                }
                            } else {
                                html! {
                                    <select
                                        class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-purple-500"
                                        value={(*selected_track_id).clone()}
                                        onchange={on_track_select}
                                        >
                                            <option value="">{"-- Select a track --"}</option>
                                            {for tracks.iter().map(|track| {
                                                html! {
                                                    <option value={track.id.clone()}>
                                                        {format!("{} - {}", track.name, track.artist_names.join(", "))}
                                                    </option>
                                                }
                                            })}
                                    </select>
                                }
                            }
                        }
                    </div>

                    <div class="flex items-center space-x-4">
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-1">
                                {"Number of recommendations:"}
                            </label>
                            <input
                                type="number"
                                min="1"
                                max="50"
                                value={limit.to_string()}
                                oninput={on_limit_change}
                                class="w-20 px-2 py-1 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-purple-500"
                            />
                        </div>

                        <div class="flex-1"></div>

                        <Button
                            onclick={get_recommendations}
                            disabled={Some(*is_loading_recommendations || selected_track_id.is_empty())}
                            variant="primary"
                        >
                            {if *is_loading_recommendations {
                                {"Finding Recommendations..."}
                            } else {
                                {"Get Recommendations"}
                            }}
                        </Button>
                    </div>
                </div>
            </Card>

            {if let Some(track) = selected_track {
                html! {
                    <Card title="Selected Track">
                        <div class={"flex items-center space-x-4"}>
                            <div class={"flex-1"}>
                                <h3 class={"text-lg font-semibold"}>{&track.name}</h3>
                                <p class={"text-gray-600"}>{"by "}{track.artist_names.join(", ")}</p>
                                <p class={"text-sm text-gray-500"}>{"Album: "}{&track.album_name}</p>

                                <div class={"grid grid-cols-3 md:grid-cols-6 gap-2 mt-3 text-xs"}>
                                    <div class={"bg-blue-100 text-blue-800 px-2 py-1 rounded text-center"}>
                                        <div>{"Energy"}</div>
                                        <div>{format!("{:.0}%", track.energy * 100.0)}</div>
                                    </div>
                                    <div class={"bg-green-100 text-green-800 px-2 py-1 rounded text-center"}>
                                        <div>{"Dance"}</div>
                                        <div>{format!("{:.0}%", track.danceability * 100.0)}</div>
                                    </div>
                                    <div class={"bg-purple-100 text-purple-800 px-2 py-1 rounded text-center"}>
                                        <div>{"Valence"}</div>
                                        <div>{format!("{:.0}%", track.valence * 100.0)}</div>
                                    </div>
                                    <div class={"bg-yellow-100 text-yellow-800 px-2 py-1 rounded text-center"}>
                                        <div>{"Tempo"}</div>
                                        <div>{format!("{:.0}", track.tempo)}</div>
                                    </div>
                                    <div class={"bg-indigo-100 text-indigo-800 px-2 py-1 rounded text-center"}>
                                        <div>{"Acoustic"}</div>
                                        <div>{format!("{:.0}%", track.acousticness * 100.0)}</div>
                                    </div>
                                    <div class={"bg-pink-100 text-pink-800 px-2 py-1 rounded text-center"}>
                                        <div>{"Popular"}</div>
                                        <div>{track.popularity}</div>
                                    </div>
                                </div>
                            </div>
                            
                            {if let Some(preview_url) = &track.preview_url {
                                if !preview_url.is_empty() {
                                    html! {
                                        <audio controls={true} class={"w-64"}>
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
                    </Card>
                }
            } else {
                html! {}
            }}

            {if !recommendations.is_empty() {
                html! {
                    <Card title={format!("Recommended Tracks ({})", recommendations.len())}>
                        <div class={"space-y-3"}>
                            {for recommendations.iter().enumerate().map(|(index, track)| {
                                html! {
                                    <div class={"flex items-center space-x-4 p-3 bg-gray-50 rounded-lg"}>
                                        <div class={"flex-shrink-0 w-8 h-8 bg-purple-100 text-purple-800 rounded-full flex items-center justify-center text-sm font-medium"}>
                                            {index + 1}
                                        </div>

                                        <div class={"flex-1 min-w-0"}>
                                            <h4 class={"font-medium text-gray-900 truncate"}>{&track.name}</h4>
                                            <p class={"text-sm text-gray-600 truncate"}>
                                                {"by "}{track.artist_names.join(", ")}
                                            </p>
                                            <p class={"text-xs text-gray-500"}>{&track.album_name}</p>
                                        </div>

                                        <div class={"flex space-x-1 text-xs"}>
                                            <span class={"bg-blue-100 text-blue-800 px-2 py-1 rounded"}>
                                                {"E: "}{format!("{:.0}%", track.energy * 100.0)}
                                            </span>
                                            <span class={"bg-green-100 text-green-800 px-2 py-1 rounded"}>
                                                {"D: "}{format!("{:.0}%", track.danceability * 100.0)}
                                            </span>
                                            <span class={"bg-purple-100 text-purple-800 px-2 py-1 rounded"}>
                                                {"V: "}{format!("{:.0}%", track.valence * 100.0)}
                                            </span>
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
                    </Card>
                }
            } else {
                html! {}
            }}
        </div>
    }
}
