//! Token sampling strategies

use rand::Rng;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

static DEFAULT_TOP_K: AtomicUsize = AtomicUsize::new(0);
// f32 config stored as raw bits so the defaults are const-initializable.
static DEFAULT_TEMPERATURE: AtomicU32 = AtomicU32::new((0.7f32).to_bits());
static DEFAULT_TOP_P: AtomicU32 = AtomicU32::new((0.9f32).to_bits());

/// Set the process-wide default top-k (installed once from cynapse.toml
/// `[sampling] top_k` via `cynapse_engine::set_sampling`).
pub fn set_default_top_k(k: usize) {
    DEFAULT_TOP_K.store(k, Ordering::Relaxed);
}

pub fn default_top_k() -> usize {
    DEFAULT_TOP_K.load(Ordering::Relaxed)
}

/// Set the process-wide default sampling (temperature, top-p).
///
/// Installed once from cynapse.toml `[sampling]` (`temperature`, `top_p`)
/// via `cynapse_engine::set_sampling`. Until then the defaults are
/// (0.7, 0.9) — see [`default_sampling`].
pub fn set_default_sampling(temperature: f32, top_p: f32) {
    DEFAULT_TEMPERATURE.store(temperature.to_bits(), Ordering::Relaxed);
    DEFAULT_TOP_P.store(top_p.to_bits(), Ordering::Relaxed);
}

/// Process-wide `(temperature, top_p)` defaults: `(0.7, 0.9)` until
/// [`set_default_sampling`] installs cynapse.toml `[sampling]` values.
///
/// `temperature <= 0.0` still selects greedy argmax in
/// [`sample_top_p_top_k`].
pub fn default_sampling() -> (f32, f32) {
    let temperature = f32::from_bits(DEFAULT_TEMPERATURE.load(Ordering::Relaxed));
    let top_p = f32::from_bits(DEFAULT_TOP_P.load(Ordering::Relaxed));
    (temperature, top_p)
}

/// Resolve the top-k to use for one sampling call.
///
/// Resolution order: `LEAFCUTTER_TOP_K` env override > configured default
/// (`set_default_top_k`) > 0 (disabled). Single source of truth shared by
/// [`sample_top_p`] and by callers that need the resolved value explicitly
/// (e.g. `StreamingOrnith::generate_with_stop`).
pub fn resolve_top_k() -> usize {
    std::env::var("LEAFCUTTER_TOP_K")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(default_top_k)
}

/// Sample next token using temperature + top-k + top-p (nucleus) sampling.
///
/// Matches Ollama's sampler order: temperature scale → sort → top-k filter
/// → top-p filter → renormalize → sample.  When `top_k == 0` the top-k
/// stage is skipped (matching Ollama's `top_k: 0` semantics).
///
/// top-k resolution order: `LEAFCUTTER_TOP_K` env override > configured
/// default (`set_default_top_k`) > 0 (disabled) — see [`resolve_top_k`].
pub fn sample_top_p(logits: &[f32], temperature: f32, top_p: f32) -> usize {
    sample_top_p_top_k(logits, temperature, top_p, resolve_top_k())
}

pub fn sample_top_p_top_k(logits: &[f32], temperature: f32, top_p: f32, top_k: usize) -> usize {
    if temperature <= 0.0 {
        // Greedy: pick highest logit (use total_cmp to handle NaN safely)
        return logits.iter().enumerate().max_by(|(_, a), (_, b)| a.total_cmp(b)).map(|(i, _)| i).unwrap_or(0);
    }

    // Apply temperature
    let mut probs: Vec<(usize, f32)> = logits.iter().enumerate().map(|(i, &v)| (i, v / temperature)).collect();

    // Softmax
    let max_logit = probs.iter().map(|(_, v)| *v).fold(f32::NEG_INFINITY, f32::max);
    let exp_sum: f32 = probs.iter().map(|(_, v)| (v - max_logit).exp()).sum();
    for (_, v) in probs.iter_mut() {
        *v = (*v - max_logit).exp() / exp_sum;
    }

    // Sort by probability descending (handle NaN safely)
    probs.sort_by(|(_, a), (_, b)| {
        b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal)
    });

    // Top-k filtering: keep only the k highest-probability tokens.
    if top_k > 0 && top_k < probs.len() {
        probs.truncate(top_k);
    }

    // Top-p filtering
    let mut cumsum = 0.0f32;
    let cutoff_idx = probs.iter().position(|(_, p)| {
        cumsum += p;
        cumsum > top_p
    }).unwrap_or(probs.len() - 1);

    let filtered: Vec<(usize, f32)> = probs[..=cutoff_idx].to_vec();

    // Renormalize and sample
    let sum: f32 = filtered.iter().map(|(_, p)| p).sum();
    let mut rng = rand::thread_rng();
    let rand_val: f32 = rng.gen::<f32>() * sum;

    let mut cum = 0.0f32;
    for (idx, p) in &filtered {
        cum += *p;
        if rand_val <= cum {
            return *idx;
        }
    }

    filtered.last().map(|(idx, _)| *idx).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greedy() {
        let logits = vec![0.1, 0.5, 0.3];
        let token = sample_top_p(&logits, 0.0, 0.9);
        assert_eq!(token, 1);
    }

    #[test]
    fn test_temperature() {
        let logits = vec![1.0, 2.0, 3.0];
        let token = sample_top_p(&logits, 1.0, 1.0);
        // Should usually pick index 2, but with randomness it's probabilistic
        assert!(token < logits.len());
    }

    #[test]
    fn test_default_sampling_defaults_and_roundtrip() {
        // Process-wide defaults are (0.7, 0.9) until cynapse.toml installs
        // real config via set_default_sampling. Single test owns the static.
        let (temperature, top_p) = default_sampling();
        assert_eq!((temperature, top_p), (0.7, 0.9));
        set_default_sampling(0.2, 0.8);
        let (temperature, top_p) = default_sampling();
        assert_eq!((temperature, top_p), (0.2, 0.8));
        set_default_sampling(0.7, 0.9); // restore process-wide default
    }

    #[test]
    fn test_temperature_zero_is_greedy_argmax() {
        let logits = vec![0.1, 0.5, 0.3];
        // temperature 0.0 must stay greedy regardless of top-p/top-k
        assert_eq!(sample_top_p_top_k(&logits, 0.0, 0.9, 0), 1);
        assert_eq!(sample_top_p_top_k(&logits, 0.0, 0.9, 1), 1);
    }

    #[test]
    fn test_resolve_top_k_matches_env_or_default() {
        let expected = std::env::var("LEAFCUTTER_TOP_K")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(default_top_k);
        assert_eq!(resolve_top_k(), expected);
    }
}
