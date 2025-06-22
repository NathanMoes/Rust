use yew::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::services::api::ApiService;
use crate::types::Artist;
use crate::components::common::{Alert, Loading, Card};

#[function_component(Artists)]
pub fn artists() -> Html {
    let artists = use_state(|| Vec::<Artist>::new());
    let is_loading = use_state(|| false);
    let error_message = use_state(|| None::<String>);

    // Load artists on component mount
    {
        let artists = artists.clone();
        let is_loading = is_loading.clone();
        let error_message = error_message.clone();
        
        use_effect_with((), move |_| {
            spawn_local(async move {
                is_loading.set(true);
                match ApiService::get_artists().await {
                    Ok(data) => {
                        artists.set(data);
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

    let refresh_artists = {
        let artists = artists.clone();
        let is_loading = is_loading.clone();
        let error_message = error_message.clone();
        
        Callback::from(move |_| {
            let artists = artists.clone();
            let is_loading = is_loading.clone();
            let error_message = error_message.clone();
            
            spawn_local(async move {
                is_loading.set(true);
                match ApiService::get_artists().await {
                    Ok(data) => {
                        artists.set(data);
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

    html! {
        <div class="max-w-6xl mx-auto">
            <div class="flex justify-between items-center mb-6">
                <h1 class="text-3xl font-bold text-gray-900">{"Artists"}</h1>
                <button
                    class="px-4 py-2 bg-purple-600 text-white rounded-md hover:bg-purple-700 disabled:bg-purple-300"
                    onclick={refresh_artists}
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

            if *is_loading && artists.is_empty() {
                <Loading message={Some("Loading artists...".to_string())} />
            } else if artists.is_empty() {
                <Card title="No Artists Found">
                    <p class="text-gray-600">
                        {"No artists have been imported yet. Import a Spotify playlist to see artists here."}
                    </p>
                </Card>
            } else {
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                    {for artists.iter().map(|artist| {
                        html! {
                            <div class="bg-white rounded-lg shadow-md p-6 hover:shadow-lg transition-shadow">
                                <div class="flex items-start space-x-4">
                                    {if let Some(image_url) = &artist.image_url {
                                        if !image_url.is_empty() {
                                            html! {
                                                <img 
                                                    src={image_url.clone()} 
                                                    alt={format!("{} image", artist.name)}
                                                    class="w-16 h-16 rounded-full object-cover"
                                                />
                                            }
                                        } else {
                                            html! {
                                                <div class="w-16 h-16 rounded-full bg-gray-200 flex items-center justify-center">
                                                    <span class="text-2xl">{"🎤"}</span>
                                                </div>
                                            }
                                        }
                                    } else {
                                        html! {
                                            <div class="w-16 h-16 rounded-full bg-gray-200 flex items-center justify-center">
                                                <span class="text-2xl">{"🎤"}</span>
                                            </div>
                                        }
                                    }}
                                    
                                    <div class="flex-1 min-w-0">
                                        <h3 class="text-lg font-semibold text-gray-900 truncate">
                                            {&artist.name}
                                        </h3>
                                        
                                        <div class="mt-2 space-y-1">
                                            <div class="flex items-center text-sm text-gray-600">
                                                <span class="font-medium">{"Popularity:"}</span>
                                                <span class="ml-1">{artist.popularity}{"/100"}</span>
                                            </div>
                                            
                                            <div class="flex items-center text-sm text-gray-600">
                                                <span class="font-medium">{"Followers:"}</span>
                                                <span class="ml-1">{artist.followers}</span>
                                            </div>
                                        </div>
                                        
                                        {if !artist.genres.is_empty() {
                                            html! {
                                                <div class="mt-3">
                                                    <div class="flex flex-wrap gap-1">
                                                        {for artist.genres.iter().take(3).map(|genre| {
                                                            html! {
                                                                <span class="px-2 py-1 bg-purple-100 text-purple-800 text-xs rounded-full">
                                                                    {genre}
                                                                </span>
                                                            }
                                                        })}
                                                        {if artist.genres.len() > 3 {
                                                            html! {
                                                                <span class="px-2 py-1 bg-gray-100 text-gray-600 text-xs rounded-full">
                                                                    {format!("+{} more", artist.genres.len() - 3)}
                                                                </span>
                                                            }
                                                        } else {
                                                            html! {}
                                                        }}
                                                    </div>
                                                </div>
                                            }
                                        } else {
                                            html! {}
                                        }}
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
