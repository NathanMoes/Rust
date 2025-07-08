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
    let hovered_node = use_state(|| None::<String>);
    
    let graph_data = props.data.clone();
    let search_query = props.search_query.clone();
    
    // Calculate node positions using an improved force-directed algorithm
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

    let node_hover = {
        let hovered_node = hovered_node.clone();
        Callback::from(move |node_id: Option<String>| {
            hovered_node.set(node_id);
        })
    };

    html! {
        <div class="graph-visualization neo4j-style" style={format!("width: {}px; height: {}px;", props.width, props.height)}>
            // Enhanced SVG-based graph visualization
            <div class="relative rounded-lg bg-gradient-to-br from-gray-900 to-gray-800 overflow-hidden shadow-2xl border border-gray-700" style={format!("width: {}px; height: {}px;", props.width, props.height)}>
                
                <svg 
                    width={props.width.to_string()} 
                    height={props.height.to_string()}
                    class="absolute inset-0"
                    xmlns="http://www.w3.org/2000/svg"
                >
                    // Define gradients and patterns
                    <defs>
                        <radialGradient id="nodeGradientTrack" cx="30%" cy="30%">
                            <stop offset="0%" style="stop-color:#60A5FA;stop-opacity:1" />
                            <stop offset="100%" style="stop-color:#3B82F6;stop-opacity:1" />
                        </radialGradient>
                        <radialGradient id="nodeGradientArtist" cx="30%" cy="30%">
                            <stop offset="0%" style="stop-color:#F87171;stop-opacity:1" />
                            <stop offset="100%" style="stop-color:#EF4444;stop-opacity:1" />
                        </radialGradient>
                        <radialGradient id="nodeGradientAlbum" cx="30%" cy="30%">
                            <stop offset="0%" style="stop-color:#FBBF24;stop-opacity:1" />
                            <stop offset="100%" style="stop-color:#F59E0B;stop-opacity:1" />
                        </radialGradient>
                        <filter id="glow">
                            <feGaussianBlur stdDeviation="3" result="coloredBlur"/>
                            <feMerge> 
                                <feMergeNode in="coloredBlur"/>
                                <feMergeNode in="SourceGraphic"/>
                            </feMerge>
                        </filter>
                        <filter id="shadow">
                            <feDropShadow dx="2" dy="2" stdDeviation="3" flood-opacity="0.3"/>
                        </filter>
                    </defs>
                    
                    // Render curved connections
                    {for graph_data.edges.iter().map(|edge| {
                        if let (Some(source_pos), Some(target_pos)) = (
                            positions.get(&edge.source),
                            positions.get(&edge.target)
                        ) {
                            let is_highlighted = (*hovered_node).as_ref()
                                .map(|h| h == &edge.source || h == &edge.target)
                                .unwrap_or(false);
                            
                            let (stroke_color, stroke_width) = match edge.relationship.as_str() {
                                "PERFORMED" => ("#10B981", if is_highlighted { "3" } else { "2" }),
                                "CONTAINS" => ("#3B82F6", if is_highlighted { "3" } else { "2" }),
                                _ => ("#6B7280", if is_highlighted { "3" } else { "2" }),
                            };
                            
                            // Create curved path
                            let mid_x = (source_pos.0 + target_pos.0) / 2.0;
                            let mid_y = (source_pos.1 + target_pos.1) / 2.0;
                            let offset = 30.0; // Curve amount
                            
                            let path = format!(
                                "M {} {} Q {} {} {} {}",
                                source_pos.0, source_pos.1,
                                mid_x, mid_y - offset,
                                target_pos.0, target_pos.1
                            );
                            
                            html! {
                                <g class="edge-group">
                                    <path
                                        d={path.clone()}
                                        stroke={stroke_color}
                                        stroke-width={stroke_width}
                                        fill="none"
                                        opacity={if is_highlighted { "1.0" } else { "0.7" }}
                                        class="transition-all duration-300"
                                        filter={if is_highlighted { "url(#glow)" } else { "" }}
                                    />
                                    // Add arrow marker
                                    <defs>
                                        <marker id={format!("arrowhead-{}", edge.relationship)} markerWidth="10" markerHeight="7" 
                                               refX="9" refY="3.5" orient="auto">
                                            <polygon points="0 0, 10 3.5, 0 7" fill={stroke_color} opacity="0.8" />
                                        </marker>
                                    </defs>
                                    <path
                                        d={path}
                                        stroke={stroke_color}
                                        stroke-width={stroke_width}
                                        fill="none"
                                        opacity={if is_highlighted { "1.0" } else { "0.7" }}
                                        marker-end={format!("url(#arrowhead-{})", edge.relationship)}
                                        class="transition-all duration-300"
                                    />
                                    // Relationship label
                                    <text
                                        x={mid_x.to_string()}
                                        y={(mid_y - offset - 5.0).to_string()}
                                        text-anchor="middle"
                                        class="text-xs font-medium fill-gray-300"
                                        opacity={if is_highlighted { "1.0" } else { "0.6" }}
                                    >
                                        {&edge.relationship}
                                    </text>
                                </g>
                            }
                        } else {
                            html! {}
                        }
                    })}
                    
                    // Render enhanced nodes
                    {for graph_data.nodes.iter().map(|node| {
                        if let Some(pos) = positions.get(&node.id) {
                            let (fill_url, size, icon) = match node.node_type.as_str() {
                                "track" => ("url(#nodeGradientTrack)", 20.0, "♪"),
                                "artist" => ("url(#nodeGradientArtist)", 30.0, "👤"),
                                "album" => ("url(#nodeGradientAlbum)", 25.0, "💿"),
                                _ => ("url(#nodeGradientTrack)", 16.0, "?"),
                            };
                            
                            let is_selected = (*selected_node).as_ref() == Some(&node.id);
                            let is_hovered = (*hovered_node).as_ref() == Some(&node.id);
                            let is_highlighted = search_query.as_ref()
                                .map(|q| node.label.to_lowercase().contains(&q.to_lowercase()))
                                .unwrap_or(false);
                            
                            let opacity = if search_query.is_some() && !is_highlighted { 0.3 } else { 1.0 };
                            let node_size = if is_hovered { size * 1.2 } else { size };
                            
                            let node_id = node.id.clone();
                            let node_click_callback = node_click.clone();
                            let node_hover_callback = node_hover.clone();
                            
                            html! {
                                <g class="node-group transition-all duration-300">
                                    // Node shadow/glow
                                    {if is_selected || is_hovered {
                                        html! {
                                            <circle
                                                cx={pos.0.to_string()}
                                                cy={pos.1.to_string()}
                                                r={(node_size + 8.0).to_string()}
                                                fill="none"
                                                stroke={if is_selected { "#A855F7" } else { "#E5E7EB" }}
                                                stroke-width={if is_selected { "3" } else { "2" }}
                                                opacity="0.8"
                                                filter="url(#glow)"
                                            />
                                        }
                                    } else {
                                        html! {}
                                    }}
                                    
                                    // Main node circle
                                    <circle
                                        cx={pos.0.to_string()}
                                        cy={pos.1.to_string()}
                                        r={node_size.to_string()}
                                        fill={fill_url}
                                        stroke="#FFFFFF"
                                        stroke-width="2"
                                        opacity={opacity.to_string()}
                                        filter="url(#shadow)"
                                        class="cursor-pointer transition-all duration-300 hover:brightness-110"
                                        onclick={
                                            let node_id = node_id.clone();
                                            move |_| node_click_callback.emit(node_id.clone())
                                        }
                                        onmouseenter={
                                            let node_id = node.id.clone();
                                            let callback = node_hover_callback.clone();
                                            move |_| callback.emit(Some(node_id.clone()))
                                        }
                                        onmouseleave={
                                            let callback = node_hover_callback.clone();
                                            move |_| callback.emit(None)
                                        }
                                    />
                                    
                                    // Node icon (for larger nodes)
                                    {if node_size >= 20.0 {
                                        html! {
                                            <text
                                                x={pos.0.to_string()}
                                                y={(pos.1 + 4.0).to_string()}
                                                text-anchor="middle"
                                                class="fill-white text-sm font-bold pointer-events-none"
                                                opacity={opacity.to_string()}
                                            >
                                                {icon}
                                            </text>
                                        }
                                    } else {
                                        html! {}
                                    }}
                                    
                                    // Node label
                                    <text
                                        x={pos.0.to_string()}
                                        y={(pos.1 + node_size + 15.0).to_string()}
                                        text-anchor="middle"
                                        class="fill-gray-100 text-xs font-medium pointer-events-none"
                                        opacity={opacity.to_string()}
                                    >
                                        {if node.label.len() > 15 {
                                            format!("{}...", &node.label[..12])
                                        } else {
                                            node.label.clone()
                                        }}
                                    </text>
                                    
                                    // Type badge
                                    <rect
                                        x={(pos.0 - 15.0).to_string()}
                                        y={(pos.1 + node_size + 20.0).to_string()}
                                        width="30"
                                        height="12"
                                        rx="6"
                                        fill="rgba(0,0,0,0.7)"
                                        opacity={opacity.to_string()}
                                    />
                                    <text
                                        x={pos.0.to_string()}
                                        y={(pos.1 + node_size + 29.0).to_string()}
                                        text-anchor="middle"
                                        class="fill-gray-300 text-xs font-medium pointer-events-none"
                                        opacity={opacity.to_string()}
                                    >
                                        {node.node_type.chars().next().unwrap_or('?').to_uppercase().to_string()}
                                    </text>
                                </g>
                            }
                        } else {
                            html! {}
                        }
                    })}
                </svg>
            </div>
            
            // Enhanced Legend with Neo4j styling
            <div class="mt-6 p-4 bg-gray-800 rounded-lg border border-gray-600">
                <h3 class="text-sm font-semibold text-gray-200 mb-3">{"Legend"}</h3>
                <div class="grid grid-cols-2 md:grid-cols-5 gap-4 text-sm">
                    <div class="flex items-center space-x-2">
                        <div class="w-4 h-4 rounded-full bg-gradient-to-br from-blue-400 to-blue-600 shadow-lg border border-white"></div>
                        <span class="text-gray-300">{"Tracks ♪"}</span>
                    </div>
                    <div class="flex items-center space-x-2">
                        <div class="w-6 h-6 rounded-full bg-gradient-to-br from-red-400 to-red-600 shadow-lg border border-white"></div>
                        <span class="text-gray-300">{"Artists 👤"}</span>
                    </div>
                    <div class="flex items-center space-x-2">
                        <div class="w-5 h-5 rounded-full bg-gradient-to-br from-yellow-400 to-yellow-600 shadow-lg border border-white"></div>
                        <span class="text-gray-300">{"Albums 💿"}</span>
                    </div>
                    <div class="flex items-center space-x-2">
                        <div class="flex items-center">
                            <div class="w-8 h-0.5 bg-green-500 rounded-full"></div>
                            <div class="w-0 h-0 border-l-4 border-l-green-500 border-t-2 border-t-transparent border-b-2 border-b-transparent ml-1"></div>
                        </div>
                        <span class="text-gray-300">{"PERFORMED"}</span>
                    </div>
                    <div class="flex items-center space-x-2">
                        <div class="flex items-center">
                            <div class="w-8 h-0.5 bg-blue-500 rounded-full"></div>
                            <div class="w-0 h-0 border-l-4 border-l-blue-500 border-t-2 border-t-transparent border-b-2 border-b-transparent ml-1"></div>
                        </div>
                        <span class="text-gray-300">{"CONTAINS"}</span>
                    </div>
                </div>
                
                // Interactive hints
                <div class="mt-3 text-xs text-gray-400 border-t border-gray-700 pt-3">
                    <div class="flex flex-wrap gap-4">
                        <span>{"💡 Click nodes to view details"}</span>
                        <span>{"🔍 Hover for highlights"}</span>
                        <span>{"🎯 Search to filter nodes"}</span>
                    </div>
                </div>
            </div>
            
            // Enhanced Selected node details with Neo4j styling
            {if let Some(selected_id) = (*selected_node).clone() {
                if let Some(node) = graph_data.nodes.iter().find(|n| n.id == selected_id) {
                    let (icon, gradient_class) = match node.node_type.as_str() {
                        "track" => ("♪", "from-blue-600 to-blue-800"),
                        "artist" => ("👤", "from-red-600 to-red-800"),
                        "album" => ("💿", "from-yellow-600 to-yellow-800"),
                        _ => ("?", "from-gray-600 to-gray-800"),
                    };
                    
                    html! {
                        <div class="mt-6 bg-gray-800 rounded-lg border border-gray-600 shadow-2xl overflow-hidden">
                            // Header with gradient background
                            <div class={format!("p-4 bg-gradient-to-r {}", gradient_class)}>
                                <div class="flex items-center space-x-3">
                                    <span class="text-2xl">{icon}</span>
                                    <div>
                                        <h3 class="font-bold text-white text-lg">
                                            {&node.label}
                                        </h3>
                                        <span class="px-2 py-1 text-xs bg-black bg-opacity-30 text-white rounded-full uppercase tracking-wide">
                                            {&node.node_type}
                                        </span>
                                    </div>
                                </div>
                            </div>
                            
                            // Content with properties
                            <div class="p-4 space-y-3">
                                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                    // Basic info
                                    <div class="space-y-2">
                                        <div class="text-gray-400 text-sm uppercase tracking-wide">{"Properties"}</div>
                                        
                                        <div class="bg-gray-900 p-3 rounded border border-gray-700">
                                            <div class="space-y-2 text-sm">
                                                <div class="flex justify-between">
                                                    <span class="text-gray-400">{"ID:"}</span>
                                                    <code class="text-green-400 text-xs bg-gray-800 px-2 py-1 rounded">{&node.id}</code>
                                                </div>
                                                <div class="flex justify-between">
                                                    <span class="text-gray-400">{"Name:"}</span>
                                                    <span class="text-white font-medium">{&node.properties.name}</span>
                                                </div>
                                                
                                                {if let Some(popularity) = node.properties.popularity {
                                                    html! {
                                                        <div class="flex justify-between items-center">
                                                            <span class="text-gray-400">{"Popularity:"}</span>
                                                            <div class="flex items-center space-x-2">
                                                                <span class="text-white">{format!("{}%", popularity)}</span>
                                                                <div class="w-16 h-2 bg-gray-700 rounded-full overflow-hidden">
                                                                    <div 
                                                                        class="h-full bg-gradient-to-r from-green-500 to-green-400 rounded-full transition-all duration-300"
                                                                        style={format!("width: {}%", popularity)}
                                                                    ></div>
                                                                </div>
                                                            </div>
                                                        </div>
                                                    }
                                                } else {
                                                    html! {}
                                                }}
                                            </div>
                                        </div>
                                    </div>
                                    
                                    // Additional properties
                                    <div class="space-y-2">
                                        {if let Some(artist_names) = &node.properties.artist_names {
                                            if !artist_names.is_empty() {
                                                html! {
                                                    <div>
                                                        <div class="text-gray-400 text-sm uppercase tracking-wide mb-2">{"Artists"}</div>
                                                        <div class="bg-gray-900 p-3 rounded border border-gray-700">
                                                            <div class="flex flex-wrap gap-1">
                                                                {for artist_names.iter().map(|artist| {
                                                                    html! {
                                                                        <span class="px-2 py-1 bg-red-900 bg-opacity-50 text-red-300 text-xs rounded-full border border-red-800">
                                                                            {artist}
                                                                        </span>
                                                                    }
                                                                })}
                                                            </div>
                                                        </div>
                                                    </div>
                                                }
                                            } else {
                                                html! {}
                                            }
                                        } else {
                                            html! {}
                                        }}
                                        
                                        {if let Some(album_name) = &node.properties.album_name {
                                            html! {
                                                <div>
                                                    <div class="text-gray-400 text-sm uppercase tracking-wide mb-2">{"Album"}</div>
                                                    <div class="bg-gray-900 p-3 rounded border border-gray-700">
                                                        <span class="px-2 py-1 bg-yellow-900 bg-opacity-50 text-yellow-300 text-sm rounded border border-yellow-800">
                                                            {album_name}
                                                        </span>
                                                    </div>
                                                </div>
                                            }
                                        } else {
                                            html! {}
                                        }}
                                        
                                        {if let Some(genres) = &node.properties.genres {
                                            if !genres.is_empty() {
                                                html! {
                                                    <div>
                                                        <div class="text-gray-400 text-sm uppercase tracking-wide mb-2">{"Genres"}</div>
                                                        <div class="bg-gray-900 p-3 rounded border border-gray-700">
                                                            <div class="flex flex-wrap gap-1">
                                                                {for genres.iter().map(|genre| {
                                                                    html! {
                                                                        <span class="px-2 py-1 bg-purple-900 bg-opacity-50 text-purple-300 text-xs rounded-full border border-purple-800">
                                                                            {genre}
                                                                        </span>
                                                                    }
                                                                })}
                                                            </div>
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
                                </div>
                                
                                // Audio features (for tracks)
                                {if let Some(audio_features) = &node.properties.audio_features {
                                    html! {
                                        <div class="mt-4">
                                            <div class="text-gray-400 text-sm uppercase tracking-wide mb-3">{"Audio Features"}</div>
                                            <div class="bg-gray-900 p-4 rounded border border-gray-700">
                                                <div class="grid grid-cols-2 md:grid-cols-3 gap-4">
                                                    {[
                                                        ("Energy", audio_features.energy * 100.0, "from-red-500 to-orange-500"),
                                                        ("Danceability", audio_features.danceability * 100.0, "from-green-500 to-blue-500"),
                                                        ("Valence", audio_features.valence * 100.0, "from-yellow-500 to-pink-500"),
                                                        ("Acousticness", audio_features.acousticness * 100.0, "from-purple-500 to-indigo-500"),
                                                        ("Instrumentalness", audio_features.instrumentalness * 100.0, "from-gray-500 to-gray-600"),
                                                    ].iter().map(|(name, value, gradient)| {
                                                        html! {
                                                            <div class="text-center">
                                                                <div class="text-xs text-gray-400 mb-1">{*name}</div>
                                                                <div class="w-full h-3 bg-gray-800 rounded-full overflow-hidden mb-1">
                                                                    <div 
                                                                        class={format!("h-full bg-gradient-to-r {} rounded-full transition-all duration-500", gradient)}
                                                                        style={format!("width: {}%", value)}
                                                                    ></div>
                                                                </div>
                                                                <div class="text-xs text-white font-mono">{format!("{:.0}%", value)}</div>
                                                            </div>
                                                        }
                                                    }).collect::<Html>()}
                                                    
                                                    <div class="text-center">
                                                        <div class="text-xs text-gray-400 mb-1">{"Tempo"}</div>
                                                        <div class="bg-gray-800 p-2 rounded">
                                                            <div class="text-lg font-mono text-cyan-400">{format!("{:.0}", audio_features.tempo)}</div>
                                                            <div class="text-xs text-gray-500">{"BPM"}</div>
                                                        </div>
                                                    </div>
                                                </div>
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
    
    // Enhanced layout algorithm for better visual appeal
    if data.nodes.len() <= 8 {
        // For small graphs, use a structured layout with node type grouping
        let mut tracks = Vec::new();
        let mut artists = Vec::new();
        let mut albums = Vec::new();
        
        // Group nodes by type
        for node in &data.nodes {
            match node.node_type.as_str() {
                "track" => tracks.push(node),
                "artist" => artists.push(node),
                "album" => albums.push(node),
                _ => tracks.push(node),
            }
        }
        
        let margin = 80.0;
        let center_x = width / 2.0;
        let center_y = height / 2.0;
        
        // Position artists in the center (most connected)
        if !artists.is_empty() {
            let artist_radius = 60.0;
            for (i, artist) in artists.iter().enumerate() {
                let angle = (2.0 * std::f64::consts::PI * i as f64) / artists.len() as f64;
                let x = center_x + artist_radius * angle.cos();
                let y = center_y + artist_radius * angle.sin();
                positions.insert(artist.id.clone(), (x, y));
            }
        }
        
        // Position albums around artists
        if !albums.is_empty() {
            let album_radius = 120.0;
            for (i, album) in albums.iter().enumerate() {
                let angle = (2.0 * std::f64::consts::PI * i as f64) / albums.len() as f64 + std::f64::consts::PI / 6.0;
                let x = center_x + album_radius * angle.cos();
                let y = center_y + album_radius * angle.sin();
                positions.insert(album.id.clone(), (x, y));
            }
        }
        
        // Position tracks on the outer ring
        if !tracks.is_empty() {
            let track_radius = 180.0;
            for (i, track) in tracks.iter().enumerate() {
                let angle = (2.0 * std::f64::consts::PI * i as f64) / tracks.len() as f64;
                let x = center_x + track_radius * angle.cos();
                let y = center_y + track_radius * angle.sin();
                
                // Keep within bounds
                let x = x.max(margin).min(width - margin);
                let y = y.max(margin).min(height - margin);
                
                positions.insert(track.id.clone(), (x, y));
            }
        }
    } else {
        // For larger graphs, use improved force-directed algorithm
        let center_x = width / 2.0;
        let center_y = height / 2.0;
        
        // Initialize positions randomly but with some structure
        for (i, node) in data.nodes.iter().enumerate() {
            let angle = (2.0 * std::f64::consts::PI * i as f64) / data.nodes.len() as f64;
            
            // Base radius depends on node type
            let base_radius = match node.node_type.as_str() {
                "artist" => (width.min(height) / 2.0) * 0.4, // Artists closer to center
                "album" => (width.min(height) / 2.0) * 0.6,   // Albums in middle
                "track" => (width.min(height) / 2.0) * 0.8,   // Tracks on outer ring
                _ => (width.min(height) / 2.0) * 0.7,
            };
            
            // Add some variation
            let variation = ((i as f64 * 7.0).sin() * 0.2 + 1.0) * base_radius;
            
            let x = center_x + variation * angle.cos();
            let y = center_y + variation * angle.sin();
            
            // Keep within safe bounds
            let margin = 50.0;
            let x = x.max(margin).min(width - margin);
            let y = y.max(margin).min(height - margin);
            
            positions.insert(node.id.clone(), (x, y));
        }
        
        // Simple force-directed refinement
        for _ in 0..50 { // Iterations for force simulation
            let mut forces = HashMap::new();
            
            // Initialize forces
            for node in &data.nodes {
                forces.insert(node.id.clone(), (0.0, 0.0));
            }
            
            // Repulsive forces between all nodes
            for i in 0..data.nodes.len() {
                for j in (i + 1)..data.nodes.len() {
                    if let (Some(pos_i), Some(pos_j)) = (
                        positions.get(&data.nodes[i].id),
                        positions.get(&data.nodes[j].id)
                    ) {
                        let dx = pos_j.0 - pos_i.0;
                        let dy = pos_j.1 - pos_i.1;
                        let distance = (dx * dx + dy * dy).sqrt().max(1.0);
                        
                        let force = 1000.0 / (distance * distance);
                        let fx = force * dx / distance;
                        let fy = force * dy / distance;
                        
                        if let Some(force_i) = forces.get_mut(&data.nodes[i].id) {
                            force_i.0 -= fx;
                            force_i.1 -= fy;
                        }
                        if let Some(force_j) = forces.get_mut(&data.nodes[j].id) {
                            force_j.0 += fx;
                            force_j.1 += fy;
                        }
                    }
                }
            }
            
            // Attractive forces for connected nodes
            for edge in &data.edges {
                if let (Some(pos_source), Some(pos_target)) = (
                    positions.get(&edge.source),
                    positions.get(&edge.target)
                ) {
                    let dx = pos_target.0 - pos_source.0;
                    let dy = pos_target.1 - pos_source.1;
                    let distance = (dx * dx + dy * dy).sqrt().max(1.0);
                    
                    let ideal_distance = 100.0;
                    let force = 0.1 * (distance - ideal_distance);
                    let fx = force * dx / distance;
                    let fy = force * dy / distance;
                    
                    if let Some(force_source) = forces.get_mut(&edge.source) {
                        force_source.0 += fx;
                        force_source.1 += fy;
                    }
                    if let Some(force_target) = forces.get_mut(&edge.target) {
                        force_target.0 -= fx;
                        force_target.1 -= fy;
                    }
                }
            }
            
            // Apply forces with damping
            let damping = 0.1;
            for node in &data.nodes {
                if let (Some(pos), Some(force)) = (
                    positions.get_mut(&node.id),
                    forces.get(&node.id)
                ) {
                    pos.0 += force.0 * damping;
                    pos.1 += force.1 * damping;
                    
                    // Keep within bounds
                    let margin = 50.0;
                    pos.0 = pos.0.max(margin).min(width - margin);
                    pos.1 = pos.1.max(margin).min(height - margin);
                }
            }
        }
    }
    
    positions
}