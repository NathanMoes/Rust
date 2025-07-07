use yew::prelude::*;
use crate::types::GraphData;
use std::collections::HashMap;

#[derive(Properties, PartialEq)]
pub struct GraphVizProps {
    pub data: GraphData,
    pub width: u32,
    pub height: u32,
    pub search_query: Option<String>,
}

#[function_component(GraphViz)]
pub fn graph_viz(props: &GraphVizProps) -> Html {
    let selected_node = use_state(|| None::<String>);
    
    let graph_data = props.data.clone();
    let search_query = props.search_query.clone();
    
    // Calculate node positions using a simple force-directed algorithm
    let positions = calculate_node_positions(&graph_data, props.width as f64, props.height as f64);
    
    let node_click = {
        let selected_node = selected_node.clone();
        Callback::from(move |node_id: String| {
            if (*selected_node).as_ref() == Some(&node_id) {
                selected_node.set(None);
            } else {
                selected_node.set(Some(node_id));
            }
        })
    };

    html! {
        <div class="graph-visualization" style={format!("width: {}px; height: {}px;", props.width, props.height)}>
            // Simple graph visualization using positioned divs
            <div class="relative border rounded-lg bg-gray-50 overflow-hidden" style={format!("width: {}px; height: {}px;", props.width, props.height)}>
                
                // Render connections as simple lines (CSS-based)
                {for graph_data.edges.iter().map(|edge| {
                    if let (Some(source_pos), Some(target_pos)) = (
                        positions.get(&edge.source),
                        positions.get(&edge.target)
                    ) {
                        let line_style = format!(
                            "position: absolute; left: {}px; top: {}px; width: {}px; height: 2px; background-color: {}; transform-origin: 0 0; transform: rotate({}deg);",
                            source_pos.0,
                            source_pos.1,
                            ((target_pos.0 - source_pos.0).powi(2) + (target_pos.1 - source_pos.1).powi(2)).sqrt(),
                            match edge.relationship.as_str() {
                                "PERFORMED" => "#10b981",
                                "CONTAINS" => "#3b82f6",
                                _ => "#6b7280",
                            },
                            (target_pos.1 - source_pos.1).atan2(target_pos.0 - source_pos.0).to_degrees()
                        );
                        
                        html! {
                            <div style={line_style} class="opacity-60"></div>
                        }
                    } else {
                        html! {}
                    }
                })}
                
                // Render nodes as positioned divs
                {for graph_data.nodes.iter().map(|node| {
                    if let Some(pos) = positions.get(&node.id) {
                        let (color, size) = match node.node_type.as_str() {
                            "track" => ("bg-blue-500", 16),
                            "artist" => ("bg-red-500", 24),
                            "album" => ("bg-yellow-500", 20),
                            _ => ("bg-gray-500", 12),
                        };
                        
                        let is_selected = (*selected_node).as_ref() == Some(&node.id);
                        let is_highlighted = search_query.as_ref()
                            .map(|q| node.label.to_lowercase().contains(&q.to_lowercase()))
                            .unwrap_or(false);
                        
                        let opacity = if search_query.is_some() && !is_highlighted { "opacity-30" } else { "opacity-100" };
                        let border = if is_selected { "ring-4 ring-purple-500" } else { "border-2 border-white" };
                        
                        let node_id = node.id.clone();
                        let node_click_callback = node_click.clone();
                        
                        let node_style = format!(
                            "position: absolute; left: {}px; top: {}px; transform: translate(-50%, -50%);",
                            pos.0, pos.1
                        );
                        
                        let node_size_style = format!(
                            "{} width: {}px; height: {}px;",
                            node_style, size, size
                        );
                        
                        html! {
                            <div>
                                // Node circle
                                <div
                                    style={node_size_style}
                                    class={format!("rounded-full cursor-pointer hover:scale-110 transition-all duration-200 {} {} {}", color, opacity, border)}
                                    onclick={
                                        let node_id = node_id.clone();
                                        move |_| node_click_callback.emit(node_id.clone())
                                    }
                                    title={node.label.clone()}
                                >
                                </div>
                                
                                // Node label
                                <div
                                    style={format!("position: absolute; left: {}px; top: {}px; transform: translate(-50%, -50%);", pos.0, pos.1 + size as f64 / 2.0 + 12.0)}
                                    class={format!("text-xs font-medium text-gray-700 pointer-events-none text-center max-w-20 {}", opacity)}
                                >
                                    {if node.label.len() > 15 {
                                        format!("{}...", &node.label[..12])
                                    } else {
                                        node.label.clone()
                                    }}
                                </div>
                            </div>
                        }
                    } else {
                        html! {}
                    }
                })}
            </div>
            
            // Legend
            <div class="mt-4 flex items-center space-x-6 text-sm">
                <div class="flex items-center space-x-2">
                    <div class="w-4 h-4 bg-blue-500 rounded-full"></div>
                    <span>{"Tracks"}</span>
                </div>
                <div class="flex items-center space-x-2">
                    <div class="w-6 h-6 bg-red-500 rounded-full"></div>
                    <span>{"Artists"}</span>
                </div>
                <div class="flex items-center space-x-2">
                    <div class="w-5 h-5 bg-yellow-500 rounded-full"></div>
                    <span>{"Albums"}</span>
                </div>
                <div class="flex items-center space-x-2">
                    <div class="w-8 h-0.5 bg-green-500"></div>
                    <span>{"Performed"}</span>
                </div>
                <div class="flex items-center space-x-2">
                    <div class="w-8 h-0.5 bg-blue-500"></div>
                    <span>{"Contains"}</span>
                </div>
            </div>
            
            // Selected node details
            {if let Some(selected_id) = (*selected_node).clone() {
                if let Some(node) = graph_data.nodes.iter().find(|n| n.id == selected_id) {
                    html! {
                        <div class="mt-4 p-4 bg-white rounded-lg border shadow-sm">
                            <h3 class="font-semibold text-gray-900 mb-2">
                                {format!("{} Details", node.node_type.to_uppercase())}
                            </h3>
                            <div class="space-y-2">
                                <div><strong>{"Name: "}</strong>{&node.label}</div>
                                <div><strong>{"ID: "}</strong><code class="text-xs bg-gray-100 px-1 rounded">{&node.id}</code></div>
                                
                                {if let Some(popularity) = node.properties.popularity {
                                    html! {
                                        <div><strong>{"Popularity: "}</strong>{format!("{}%", popularity)}</div>
                                    }
                                } else {
                                    html! {}
                                }}
                                
                                {if let Some(artist_names) = &node.properties.artist_names {
                                    if !artist_names.is_empty() {
                                        html! {
                                            <div><strong>{"Artists: "}</strong>{artist_names.join(", ")}</div>
                                        }
                                    } else {
                                        html! {}
                                    }
                                } else {
                                    html! {}
                                }}
                                
                                {if let Some(album_name) = &node.properties.album_name {
                                    html! {
                                        <div><strong>{"Album: "}</strong>{album_name}</div>
                                    }
                                } else {
                                    html! {}
                                }}
                                
                                {if let Some(genres) = &node.properties.genres {
                                    if !genres.is_empty() {
                                        html! {
                                            <div><strong>{"Genres: "}</strong>{genres.join(", ")}</div>
                                        }
                                    } else {
                                        html! {}
                                    }
                                } else {
                                    html! {}
                                }}
                                
                                {if let Some(audio_features) = &node.properties.audio_features {
                                    html! {
                                        <div class="mt-2">
                                            <strong>{"Audio Features:"}</strong>
                                            <div class="grid grid-cols-2 gap-2 mt-1 text-sm">
                                                <div>{"Energy: "}{format!("{:.0}%", audio_features.energy * 100.0)}</div>
                                                <div>{"Dance: "}{format!("{:.0}%", audio_features.danceability * 100.0)}</div>
                                                <div>{"Valence: "}{format!("{:.0}%", audio_features.valence * 100.0)}</div>
                                                <div>{"Tempo: "}{format!("{:.0} BPM", audio_features.tempo)}</div>
                                            </div>
                                        </div>
                                    }
                                } else {
                                    html! {}
                                }}
                            </div>
                        </div>
                    }
                } else {
                    html! {}
                }
            } else {
                html! {}
            }}
        </div>
    }
}

fn calculate_node_positions(data: &GraphData, width: f64, height: f64) -> HashMap<String, (f64, f64)> {
    let mut positions = HashMap::new();
    
    if data.nodes.is_empty() {
        return positions;
    }
    
    // For small numbers of nodes, use a grid layout
    if data.nodes.len() <= 10 {
        let cols = (data.nodes.len() as f64).sqrt().ceil() as usize;
        let rows = (data.nodes.len() + cols - 1) / cols;
        
        let margin = 50.0;
        let usable_width = width - 2.0 * margin;
        let usable_height = height - 2.0 * margin;
        
        let cell_width = usable_width / cols as f64;
        let cell_height = usable_height / rows as f64;
        
        for (i, node) in data.nodes.iter().enumerate() {
            let col = i % cols;
            let row = i / cols;
            
            let x = margin + (col as f64 + 0.5) * cell_width;
            let y = margin + (row as f64 + 0.5) * cell_height;
            
            positions.insert(node.id.clone(), (x, y));
        }
    } else {
        // For larger numbers, use a circular layout
        let center_x = width / 2.0;
        let center_y = height / 2.0;
        let radius = (width.min(height) / 2.0) * 0.7;
        
        let node_count = data.nodes.len() as f64;
        
        for (i, node) in data.nodes.iter().enumerate() {
            let angle = (2.0 * std::f64::consts::PI * i as f64) / node_count;
            
            // Add some variation based on node type
            let type_offset = match node.node_type.as_str() {
                "track" => 0.8,
                "artist" => 1.0,
                "album" => 0.9,
                _ => 1.0,
            };
            
            let x = center_x + (radius * type_offset * angle.cos());
            let y = center_y + (radius * type_offset * angle.sin());
            
            positions.insert(node.id.clone(), (x, y));
        }
    }
    
    positions
}