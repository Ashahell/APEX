use chrono::{DateTime, Utc};
use std::collections::HashMap;

pub fn rrf_score(rank: usize, k: usize) -> f64 {
    1.0 / (k + rank) as f64
}

pub fn reciprocal_rank_fusion(
    vec_ranks: &[(String, usize)],
    bm25_ranks: &[(String, usize)],
    k: usize,
) -> Vec<(String, f64)> {
    let mut scores: HashMap<String, f64> = HashMap::new();

    for (chunk_id, rank) in vec_ranks {
        *scores.entry(chunk_id.clone()).or_default() += rrf_score(*rank, k);
    }
    for (chunk_id, rank) in bm25_ranks {
        *scores.entry(chunk_id.clone()).or_default() += rrf_score(*rank, k);
    }

    let mut merged: Vec<(String, f64)> = scores.into_iter().collect();
    merged.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    merged
}

pub fn temporal_decay(accessed_at: DateTime<Utc>, half_life_days: f64) -> f64 {
    let age_days = (Utc::now() - accessed_at).num_seconds() as f64 / 86_400.0;
    2.0_f64.powf(-age_days / half_life_days)
}

pub fn frequency_boost(access_count: u64) -> f64 {
    (1.0 + access_count as f64).ln()
}

pub fn apply_temporal_score(
    rrf_score: f64,
    accessed_at: DateTime<Utc>,
    access_count: u64,
    half_life_days: f64,
) -> f64 {
    rrf_score * temporal_decay(accessed_at, half_life_days) * frequency_boost(access_count).max(1.0)
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if magnitude_a == 0.0 || magnitude_b == 0.0 {
        return 0.0;
    }

    dot_product / (magnitude_a * magnitude_b)
}

pub fn mmr_select(
    candidates: &[(String, f64, Vec<f32>)],
    query_vec: &[f32],
    n: usize,
    lambda: f64,
) -> Vec<String> {
    if candidates.is_empty() || n == 0 {
        return Vec::new();
    }

    let mut selected: Vec<(String, Vec<f32>)> = Vec::with_capacity(n);
    let mut remaining: Vec<(String, f64, Vec<f32>)> = candidates.to_vec();

    while selected.len() < n && !remaining.is_empty() {
        let best_idx = remaining
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| {
                let a_relevance = cosine_similarity(&a.2, query_vec);
                let a_redundancy = selected
                    .iter()
                    .map(|(_, s_emb)| cosine_similarity(&a.2, s_emb))
                    .fold(0.0_f32, f32::max);
                let a_mmr = lambda * a_relevance as f64 - (1.0 - lambda) * a_redundancy as f64;

                let b_relevance = cosine_similarity(&b.2, query_vec);
                let b_redundancy = selected
                    .iter()
                    .map(|(_, s_emb)| cosine_similarity(&b.2, s_emb))
                    .fold(0.0_f32, f32::max);
                let b_mmr = lambda * b_relevance as f64 - (1.0 - lambda) * b_redundancy as f64;

                a_mmr
                    .partial_cmp(&b_mmr)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(i, _)| i);

        if let Some(idx) = best_idx {
            let chosen = remaining.remove(idx);
            selected.push((chosen.0.clone(), chosen.2.clone()));
        } else {
            break;
        }
    }

    selected.into_iter().map(|(id, _)| id).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rrf_score() {
        assert!((rrf_score(0, 60) - 1.0 / 60.0).abs() < 0.0001);
        assert!((rrf_score(1, 60) - 1.0 / 61.0).abs() < 0.0001);
        assert!((rrf_score(59, 60) - 1.0 / 119.0).abs() < 0.0001);
    }

    #[test]
    fn test_reciprocal_rank_fusion() {
        let vec_ranks = vec![
            ("doc1".to_string(), 0),
            ("doc2".to_string(), 1),
            ("doc3".to_string(), 2),
        ];
        let bm25_ranks = vec![
            ("doc2".to_string(), 0),
            ("doc3".to_string(), 1),
            ("doc4".to_string(), 2),
        ];

        let fused = reciprocal_rank_fusion(&vec_ranks, &bm25_ranks, 60);

        assert_eq!(fused[0].0, "doc2");
        assert!(fused[0].1 >= fused[1].1);
    }

    #[test]
    fn test_temporal_decay() {
        let now = Utc::now();
        let decay_at_0 = temporal_decay(now, 30.0);
        assert!((decay_at_0 - 1.0).abs() < 0.0001);

        let thirty_days_ago = now - chrono::Duration::days(30);
        let decay_at_30 = temporal_decay(thirty_days_ago, 30.0);
        assert!((decay_at_30 - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_frequency_boost() {
        let boost_0 = frequency_boost(0);
        assert!((boost_0 - 0.0).abs() < 0.0001);

        let boost_1 = frequency_boost(1);
        assert!((boost_1 - 0.6931).abs() < 0.01);

        let boost_100 = frequency_boost(100);
        assert!(boost_100 > boost_1);
    }

    #[test]
    fn test_mmr_select() {
        let candidates = vec![
            ("doc1".to_string(), 0.9, vec![1.0, 0.0, 0.0]),
            ("doc2".to_string(), 0.8, vec![0.9, 0.1, 0.0]),
            ("doc3".to_string(), 0.7, vec![0.0, 1.0, 0.0]),
            ("doc4".to_string(), 0.6, vec![0.0, 0.9, 0.1]),
        ];
        let query = vec![1.0, 0.0, 0.0];

        let selected = mmr_select(&candidates, &query, 2, 0.7);

        assert_eq!(selected.len(), 2);
        assert!(selected.contains(&"doc1".to_string()));
    }
}
