use yew::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::services::api::ApiService;
use crate::types::{GraphData, GraphNode, GraphEdge};
use crate::components::common::{Loading, Card};

#[function_component(Graph)]
pub fn graph() -> Html {
    let graph_data = use_state(|| None::<GraphData>);
    let loading = use_state(|| false);
    let error_message = use_state(|| None::<String>);
    let search_query = use_state(|| String::new());

    // Load initial graph data
    let load_graph_data = {
        let graph_data = graph_data.clone();
        let loading = loading.clone();
        let error_message = error_message.clone();
        Callback::from(move |query: Option<String>| {
            let graph_data = graph_data.clone();
            let loading = loading.clone();
            let error_message = error_message.clone();
            
            spawn_local(async move {
                loading.set(true);
                error_message.set(None);
                
                match ApiService::get_graph_data(query, Some(30)).await {
                    Ok(data) => {
                        graph_data.set(Some(data));
                        error_message.set(None);
                    }
                    Err(error) => {
                        error_message.set(Some(error));
                    }
                }
                loading.set(false);
            });
        })
    };

    // Load data on component mount
    {
        let load_graph_data = load_graph_data.clone();
        use_effect_with((), move |_| {
            load_graph_data.emit(None);
            || {}
        });
    }

    let on_search = {
        let search_query = search_query.clone();
        let load_graph_data = load_graph_data.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                let value = input.value();
                search_query.set(value.clone());
                
                // Load graph data with search query
                if value.trim().is_empty() {
                    load_graph_data.emit(None);
                } else {
                    load_graph_data.emit(Some(value));
                }
            }
        })
    };

    let on_search_keyup = {
        let search_query = search_query.clone();
        let load_graph_data = load_graph_data.clone();
        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Enter" {
                let query = (*search_query).clone();
                if query.trim().is_empty() {
                    load_graph_data.emit(None);
                } else {
                    load_graph_data.emit(Some(query));
                }
            }
        })
    };

    let clear_search = {
        let search_query = search_query.clone();
        let load_graph_data = load_graph_data.clone();
        Callback::from(move |_| {
            search_query.set(String::new());
            load_graph_data.emit(None);
        })
    };

    html! {
        <div class="max-w-7xl mx-auto">
            <div class="flex justify-between items-center mb-6">
                <h1 class="text-2xl font-bold text-gray-900">{"Music Graph Explorer"}</h1>
                <div class="flex items-center space-x-4">
                    <div class="relative">
                        <input
                            type="text"
                            placeholder="Search tracks, artists, or albums..."
                            class="w-64 px-4 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-purple-500"
                            value={(*search_query).clone()}
                            oninput={on_search}
                            onkeyup={on_search_keyup}
                        />
                        if !(*search_query).is_empty() {
                            <button
                                class="absolute right-2 top-2 text-gray-400 hover:text-gray-600"
                                onclick={clear_search}
                            >
                                <svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
                                </svg>
                            </button>
                        }
                    </div>
                </div>
            </div>

            if let Some(error) = (*error_message).clone() {
                <div class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded mb-4">
                    {error}
                </div>
            }

            if *loading {
                <Loading message={Some("Loading graph data...".to_string())} />
            } else if let Some(data) = (*graph_data).clone() {
                <div class="space-y-6">
                    // Graph Statistics
                    <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                        <Card title={"Nodes".to_string()}>
                            <div class="text-3xl font-bold text-purple-600">{data.nodes.len()}</div>
                            <div class="text-sm text-gray-600">{"Total nodes in graph"}</div>
                        </Card>
                        <Card title={"Relationships".to_string()}>
                            <div class="text-3xl font-bold text-green-600">{data.edges.len()}</div>
                            <div class="text-sm text-gray-600">{"Total relationships"}</div>
                        </Card>
                        <Card title={"Node Types".to_string()}>
                            <div class="space-y-1">
                                {
                                    data.nodes.iter()
                                        .fold(std::collections::HashMap::new(), |mut acc, node| {
                                            *acc.entry(node.node_type.clone()).or_insert(0) += 1;
                                            acc
                                        })
                                        .into_iter()
                                        .map(|(node_type, count)| {
                                            let color = match node_type.as_str() {
                                                "track" => "text-blue-600",
                                                "artist" => "text-red-600",
                                                "album" => "text-yellow-600",
                                                _ => "text-gray-600",
                                            };
                                            html! {
                                                <div class="flex justify-between">
                                                    <span class={format!("capitalize {}", color)}>{node_type}</span>
                                                    <span class="font-semibold">{count}</span>
                                                </div>
                                            }
                                        })
                                        .collect::<Html>()
                                }
                            </div>
                        </Card>
                    </div>

                    // Graph Visualization Placeholder
                    <Card title={"Graph Visualization".to_string()}>
                        <div id="graph-container" class="w-full h-96 bg-gray-50 rounded-lg flex items-center justify-center">
                            <div class="text-center">
                                <svg class="mx-auto h-12 w-12 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
                                </svg>
                                <h3 class="mt-2 text-sm font-medium text-gray-900">{"Interactive Graph Visualization"}</h3>
                                <p class="mt-1 text-sm text-gray-500">{"Force-directed graph showing relationships between tracks, artists, and albums"}</p>
                            </div>
                        </div>
                    </Card>

                    // Node Details
                    <Card title={"Graph Nodes".to_string()}>
                        <div class="space-y-4">
                            {for data.nodes.iter().map(|node| render_node_card(node))}
                        </div>
                    </Card>

                    // Edge Details
                    <Card title={"Relationships".to_string()}>
                        <div class="space-y-2">
                            {for data.edges.iter().map(|edge| render_edge_card(edge))}
                        </div>
                    </Card>
                </div>
            } else {
                <div class="text-center py-8">
                    <div class="text-gray-500">{"No graph data available. Try searching for tracks, artists, or albums."}</div>
                </div>
            }
        </div>
    }
}

fn render_node_card(node: &GraphNode) -> Html {
    let (icon, color) = match node.node_type.as_str() {
        "track" => ("🎵", "border-blue-200 bg-blue-50"),
        "artist" => ("👤", "border-red-200 bg-red-50"),
        "album" => ("💿", "border-yellow-200 bg-yellow-50"),
        _ => ("❓", "border-gray-200 bg-gray-50"),
    };

    html! {
        <div class={format!("p-4 rounded-lg border-2 {}", color)}>
            <div class="flex items-start justify-between">
                <div class="flex-1">
                    <div class="flex items-center space-x-2">
                        <span class="text-lg">{icon}</span>
                        <h3 class="font-semibold text-gray-900">{&node.label}</h3>
                        <span class="px-2 py-1 text-xs bg-white rounded-full border capitalize">
                            {&node.node_type}
                        </span>
                    </div>
                    
                    <div class="mt-2 space-y-1">
                        if let Some(popularity) = node.properties.popularity {
                            <div class="text-sm text-gray-600">
                                {format!("Popularity: {}%", popularity)}
                            </div>
                        }
                        
                        if let Some(artist_names) = &node.properties.artist_names {
                            <div class="text-sm text-gray-600">
                                {format!("Artists: {}", artist_names.join(", "))}
                            </div>
                        }
                        
                        if let Some(album_name) = &node.properties.album_name {
                            <div class="text-sm text-gray-600">
                                {format!("Album: {}", album_name)}
                            </div>
                        }
                        
                        if let Some(genres) = &node.properties.genres {
                            if !genres.is_empty() {
                                <div class="text-sm text-gray-600">
                                    {format!("Genres: {}", genres.join(", "))}
                                </div>
                            }
                        }
                        
                        if let Some(audio_features) = &node.properties.audio_features {
                            <div class="grid grid-cols-3 gap-2 mt-2">
                                <div class="text-xs bg-white p-1 rounded">
                                    {format!("Energy: {:.0}%", audio_features.energy * 100.0)}
                                </div>
                                <div class="text-xs bg-white p-1 rounded">
                                    {format!("Dance: {:.0}%", audio_features.danceability * 100.0)}
                                </div>
                                <div class="text-xs bg-white p-1 rounded">
                                    {format!("Valence: {:.0}%", audio_features.valence * 100.0)}
                                </div>
                            </div>
                        }
                    </div>
                </div>
            </div>
        </div>
    }
}

fn render_edge_card(edge: &GraphEdge) -> Html {
    let relationship_color = match edge.relationship.as_str() {
        "PERFORMED" => "text-green-600",
        "CONTAINS" => "text-blue-600",
        _ => "text-gray-600",
    };

    html! {
        <div class="flex items-center justify-between py-2 px-3 bg-gray-50 rounded">
            <div class="flex items-center space-x-3">
                <div class="font-mono text-sm text-gray-600">{&edge.source}</div>
                <div class="flex items-center space-x-1">
                    <span class="text-gray-400">{"→"}</span>
                    <span class={format!("text-xs font-semibold {}", relationship_color)}>
                        {&edge.relationship}
                    </span>
                    <span class="text-gray-400">{"→"}</span>
                </div>
                <div class="font-mono text-sm text-gray-600">{&edge.target}</div>
            </div>
        </div>
    }
}