use crate::models::{Artist, Track};
use neo4rs::{Graph, Query};
use anyhow::Result;
use std::sync::Arc;

pub type Neo4jClient = Arc<Graph>;

pub async fn init_neo4j() -> Result<Neo4jClient> {
    let uri = std::env::var("NEO4J_URI").unwrap_or_else(|_| "bolt://localhost:7687".to_string());
    let user = std::env::var("NEO4J_USER").unwrap_or_else(|_| "neo4j".to_string());
    let password = std::env::var("NEO4J_PASSWORD").expect("NEO4J_PASSWORD must be set");

    let graph = Graph::new(uri, user, password).await?;
    
    // Create indexes for better performance
    create_indexes(&graph).await?;
    
    Ok(Arc::new(graph))
}

async fn create_indexes(graph: &Graph) -> Result<()> {
    let queries = vec![
        "CREATE INDEX artist_id_index IF NOT EXISTS FOR (a:Artist) ON (a.id)",
        "CREATE INDEX track_id_index IF NOT EXISTS FOR (t:Track) ON (t.id)",
        "CREATE INDEX album_id_index IF NOT EXISTS FOR (al:Album) ON (al.id)",
        "CREATE INDEX artist_name_index IF NOT EXISTS FOR (a:Artist) ON (a.name)",
        "CREATE INDEX track_name_index IF NOT EXISTS FOR (t:Track) ON (t.name)",
    ];

    for query_str in queries {
        let query = Query::new(query_str.to_string());
        let _ = graph.execute(query).await?;
    }

    Ok(())
}

pub async fn store_artist(graph: &Graph, artist: &Artist) -> Result<()> {
    let query = Query::new(
        "MERGE (a:Artist {id: $id})
         SET a.name = $name,
             a.genres = $genres,
             a.popularity = $popularity,
             a.followers = $followers,
             a.image_url = $image_url,
             a.updated_at = datetime()
         RETURN a".to_string()
    )
    .param("id", artist.id.clone())
    .param("name", artist.name.clone())
    .param("genres", artist.genres.clone())
    .param("popularity", artist.popularity as i64)
    .param("followers", artist.followers as i64)
    .param("image_url", artist.image_url.clone().unwrap_or_default());

    let _ = graph.execute(query).await?;
    Ok(())
}

pub async fn store_track(graph: &Graph, track: &Track) -> Result<()> {
    let query = Query::new(
        "MERGE (t:Track {id: $id})
         SET t.name = $name,
             t.duration_ms = $duration_ms,
             t.popularity = $popularity,
             t.explicit = $explicit,
             t.danceability = $danceability,
             t.energy = $energy,
             t.key = $key,
             t.loudness = $loudness,
             t.mode = $mode,
             t.speechiness = $speechiness,
             t.acousticness = $acousticness,
             t.instrumentalness = $instrumentalness,
             t.liveness = $liveness,
             t.valence = $valence,
             t.tempo = $tempo,
             t.time_signature = $time_signature,
             t.preview_url = $preview_url,
             t.updated_at = datetime()
         RETURN t".to_string()
    )
    .param("id", track.id.clone())
    .param("name", track.name.clone())
    .param("duration_ms", track.duration_ms as i64)
    .param("popularity", track.popularity as i64)
    .param("explicit", track.explicit)
    .param("danceability", track.danceability)
    .param("energy", track.energy)
    .param("key", track.key as i64)
    .param("loudness", track.loudness)
    .param("mode", track.mode as i64)
    .param("speechiness", track.speechiness)
    .param("acousticness", track.acousticness)
    .param("instrumentalness", track.instrumentalness)
    .param("liveness", track.liveness)
    .param("valence", track.valence)
    .param("tempo", track.tempo)
    .param("time_signature", track.time_signature as i64)
    .param("preview_url", track.preview_url.clone().unwrap_or_default());

    let _ = graph.execute(query).await?;

    // Create relationships with artists
    for artist_id in &track.artist_ids {
        let rel_query = Query::new(
            "MATCH (t:Track {id: $track_id}), (a:Artist {id: $artist_id})
             MERGE (a)-[:PERFORMED]->(t)".to_string()
        )
        .param("track_id", track.id.clone())
        .param("artist_id", artist_id.clone());

        let _ = graph.execute(rel_query).await?;
    }

    // Create relationship with album if it exists
    if !track.album_id.is_empty() {
        let album_query = Query::new(
            "MERGE (al:Album {id: $album_id})
             SET al.name = $album_name
             WITH al
             MATCH (t:Track {id: $track_id})
             MERGE (al)-[:CONTAINS]->(t)".to_string()
        )
        .param("album_id", track.album_id.clone())
        .param("album_name", track.album_name.clone())
        .param("track_id", track.id.clone());

        let _ = graph.execute(album_query).await?;
    }

    Ok(())
}

pub async fn get_all_artists(graph: &Graph) -> Result<Vec<Artist>> {
    let query = Query::new(
        "MATCH (a:Artist)
         RETURN a.id as id, a.name as name, a.genres as genres, 
                a.popularity as popularity, a.followers as followers, 
                a.image_url as image_url
         ORDER BY a.popularity DESC".to_string()
    );

    let mut result = graph.execute(query).await?;
    let mut artists = Vec::new();

    while let Some(row) = result.next().await? {
        artists.push(Artist {
            id: row.get::<String>("id")?,
            name: row.get::<String>("name")?,
            genres: row.get::<Vec<String>>("genres").unwrap_or_default(),
            popularity: row.get::<i64>("popularity").unwrap_or(0) as i32,
            followers: row.get::<i64>("followers").unwrap_or(0) as i32,
            image_url: row.get::<Option<String>>("image_url")?,
        });
    }

    Ok(artists)
}

pub async fn get_all_tracks(graph: &Graph) -> Result<Vec<Track>> {
    let query = Query::new(
        "MATCH (t:Track)<-[:PERFORMED]-(a:Artist)
         OPTIONAL MATCH (al:Album)-[:CONTAINS]->(t)
         RETURN t.id as id, t.name as name, 
                collect(DISTINCT a.id) as artist_ids,
                collect(DISTINCT a.name) as artist_names,
                COALESCE(al.id, '') as album_id,
                COALESCE(al.name, '') as album_name,
                t.duration_ms as duration_ms, t.popularity as popularity,
                t.explicit as explicit, t.danceability as danceability,
                t.energy as energy, t.key as key, t.loudness as loudness,
                t.mode as mode, t.speechiness as speechiness,
                t.acousticness as acousticness, t.instrumentalness as instrumentalness,
                t.liveness as liveness, t.valence as valence,
                t.tempo as tempo, t.time_signature as time_signature,
                t.preview_url as preview_url
         ORDER BY t.popularity DESC".to_string()
    );

    let mut result = graph.execute(query).await?;
    let mut tracks = Vec::new();

    while let Some(row) = result.next().await? {
        tracks.push(Track {
            id: row.get::<String>("id")?,
            name: row.get::<String>("name")?,
            artist_ids: row.get::<Vec<String>>("artist_ids").unwrap_or_default(),
            artist_names: row.get::<Vec<String>>("artist_names").unwrap_or_default(),
            album_id: row.get::<String>("album_id").unwrap_or_default(),
            album_name: row.get::<String>("album_name").unwrap_or_default(),
            duration_ms: row.get::<i64>("duration_ms").unwrap_or(0) as i32,
            popularity: row.get::<i64>("popularity").unwrap_or(0) as i32,
            explicit: row.get::<bool>("explicit").unwrap_or(false),
            danceability: row.get::<f64>("danceability").unwrap_or(0.0),
            energy: row.get::<f64>("energy").unwrap_or(0.0),
            key: row.get::<i64>("key").unwrap_or(0) as i32,
            loudness: row.get::<f64>("loudness").unwrap_or(0.0),
            mode: row.get::<i64>("mode").unwrap_or(0) as i32,
            speechiness: row.get::<f64>("speechiness").unwrap_or(0.0),
            acousticness: row.get::<f64>("acousticness").unwrap_or(0.0),
            instrumentalness: row.get::<f64>("instrumentalness").unwrap_or(0.0),
            liveness: row.get::<f64>("liveness").unwrap_or(0.0),
            valence: row.get::<f64>("valence").unwrap_or(0.0),
            tempo: row.get::<f64>("tempo").unwrap_or(0.0),
            time_signature: row.get::<i64>("time_signature").unwrap_or(4) as i32,
            preview_url: row.get::<Option<String>>("preview_url")?,
        });
    }

    Ok(tracks)
}

pub async fn get_similar_tracks(graph: &Graph, track_ids: &[String], limit: i32) -> Result<Vec<Track>> {
    let query = Query::new(
        "MATCH (seed:Track) WHERE seed.id IN $seed_ids
         MATCH (similar:Track)
         WHERE similar.id <> seed.id
         WITH similar, seed,
              abs(similar.valence - seed.valence) as valence_diff,
              abs(similar.energy - seed.energy) as energy_diff,
              abs(similar.danceability - seed.danceability) as dance_diff,
              abs(similar.tempo - seed.tempo) / 200.0 as tempo_diff
         WITH similar, 
              avg(valence_diff + energy_diff + dance_diff + tempo_diff) as similarity_score
         ORDER BY similarity_score ASC
         LIMIT $limit
         MATCH (similar)<-[:PERFORMED]-(a:Artist)
         OPTIONAL MATCH (al:Album)-[:CONTAINS]->(similar)
         RETURN similar.id as id, similar.name as name,
                collect(DISTINCT a.id) as artist_ids,
                collect(DISTINCT a.name) as artist_names,
                COALESCE(al.id, '') as album_id,
                COALESCE(al.name, '') as album_name,
                similar.duration_ms as duration_ms, similar.popularity as popularity,
                similar.explicit as explicit, similar.danceability as danceability,
                similar.energy as energy, similar.key as key, similar.loudness as loudness,
                similar.mode as mode, similar.speechiness as speechiness,
                similar.acousticness as acousticness, similar.instrumentalness as instrumentalness,
                similar.liveness as liveness, similar.valence as valence,
                similar.tempo as tempo, similar.time_signature as time_signature,
                similar.preview_url as preview_url".to_string()
    )
    .param("seed_ids", track_ids.to_vec())
    .param("limit", limit as i64);

    let mut result = graph.execute(query).await?;
    let mut tracks = Vec::new();

    while let Some(row) = result.next().await? {
        tracks.push(Track {
            id: row.get::<String>("id")?,
            name: row.get::<String>("name")?,
            artist_ids: row.get::<Vec<String>>("artist_ids").unwrap_or_default(),
            artist_names: row.get::<Vec<String>>("artist_names").unwrap_or_default(),
            album_id: row.get::<String>("album_id").unwrap_or_default(),
            album_name: row.get::<String>("album_name").unwrap_or_default(),
            duration_ms: row.get::<i64>("duration_ms").unwrap_or(0) as i32,
            popularity: row.get::<i64>("popularity").unwrap_or(0) as i32,
            explicit: row.get::<bool>("explicit").unwrap_or(false),
            danceability: row.get::<f64>("danceability").unwrap_or(0.0),
            energy: row.get::<f64>("energy").unwrap_or(0.0),
            key: row.get::<i64>("key").unwrap_or(0) as i32,
            loudness: row.get::<f64>("loudness").unwrap_or(0.0),
            mode: row.get::<i64>("mode").unwrap_or(0) as i32,
            speechiness: row.get::<f64>("speechiness").unwrap_or(0.0),
            acousticness: row.get::<f64>("acousticness").unwrap_or(0.0),
            instrumentalness: row.get::<f64>("instrumentalness").unwrap_or(0.0),
            liveness: row.get::<f64>("liveness").unwrap_or(0.0),
            valence: row.get::<f64>("valence").unwrap_or(0.0),
            tempo: row.get::<f64>("tempo").unwrap_or(0.0),
            time_signature: row.get::<i64>("time_signature").unwrap_or(4) as i32,
            preview_url: row.get::<Option<String>>("preview_url")?,
        });
    }

    Ok(tracks)
}
