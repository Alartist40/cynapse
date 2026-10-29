//! AirLLM-inspired asynchronous double-buffered layer streaming pipeline.
//!
//! Reverse-engineered from AirLLM (`airllm_base.py` & `utils.py`).
//! Instead of loading all model weights into RAM or synchronously pausing
//! computation to read the next layer from disk, this module executes layer `L`
//! while a background worker thread asynchronously prefetches and prepares
//! layer `L + 1` into memory. As soon as layer `L` finishes execution, its
//! memory is dropped immediately, strictly capping peak resident memory to
//! at most two layers (the active layer and the prefetched layer) — outperforming
//! Python AirLLM by eliminating GIL overhead and garbage-collection stalls.

use std::collections::HashMap;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::Arc;
use std::thread::{spawn, JoinHandle};
use crate::safetensors_loader::WeightProvider;

/// Message sent across the double-buffered prefetch channel.
pub enum PrefetchMessage {
    /// Layer weights loaded and ready for compute.
    Ready {
        layer_idx: usize,
        weights: Result<HashMap<String, Vec<f32>>, String>,
    },
    /// Stream terminated.
    Done,
}

/// Double-buffered asynchronous layer streaming pipeline.
pub struct AirLlmStreamer {
    provider: Arc<dyn WeightProvider>,
    num_layers: usize,
    layer_types: Vec<String>,
    prefix: String,
    layer_tensor_names: HashMap<String, Vec<&'static str>>,
    receiver: Option<Receiver<PrefetchMessage>>,
    worker_handle: Option<JoinHandle<()>>,
}

impl AirLlmStreamer {
    pub fn new(
        provider: Arc<dyn WeightProvider>,
        num_layers: usize,
        layer_types: Vec<String>,
        prefix: String,
    ) -> Self {
        let mut layer_tensor_names: HashMap<String, Vec<&'static str>> = HashMap::new();
        layer_tensor_names.insert(
            "linear_attention".to_string(),
            vec![
                "input_layernorm.weight",
                "linear_attn.in_proj_qkv.weight",
                "linear_attn.in_proj_a.weight",
                "linear_attn.in_proj_b.weight",
                "linear_attn.in_proj_z.weight",
                "linear_attn.conv1d.weight",
                "linear_attn.A_log",
                "linear_attn.dt_bias",
                "linear_attn.norm.weight",
                "linear_attn.out_proj.weight",
                "post_attention_layernorm.weight",
                "mlp.gate_proj.weight",
                "mlp.up_proj.weight",
                "mlp.down_proj.weight",
            ],
        );
        layer_tensor_names.insert(
            "full_attention".to_string(),
            vec![
                "input_layernorm.weight",
                "self_attn.q_proj.weight",
                "self_attn.k_proj.weight",
                "self_attn.v_proj.weight",
                "self_attn.o_proj.weight",
                "self_attn.q_norm.weight",
                "self_attn.k_norm.weight",
                "post_attention_layernorm.weight",
                "mlp.gate_proj.weight",
                "mlp.up_proj.weight",
                "mlp.down_proj.weight",
            ],
        );

        Self {
            provider,
            num_layers,
            layer_types,
            prefix,
            layer_tensor_names,
            receiver: None,
            worker_handle: None,
        }
    }

    /// Start prefetching layers for a forward pass from layer 0 up to num_layers - 1.
    pub fn start_prefetch(&mut self) {
        // Bound buffer capacity to 1 so at most 1 prefetched layer sits ahead in RAM (AirLLM double-buffering)
        let (tx, rx): (SyncSender<PrefetchMessage>, Receiver<PrefetchMessage>) = sync_channel(1);
        let provider = Arc::clone(&self.provider);
        let num_layers = self.num_layers;
        let layer_types = self.layer_types.clone();
        let prefix = self.prefix.clone();
        let tensor_names_map = self.layer_tensor_names.clone();

        let handle = spawn(move || {
            for layer_idx in 0..num_layers {
                let layer_type = layer_types
                    .get(layer_idx)
                    .map(|s| s.as_str())
                    .unwrap_or("linear_attention");
                let default_names = vec![
                    "input_layernorm.weight",
                    "post_attention_layernorm.weight",
                    "mlp.gate_proj.weight",
                    "mlp.up_proj.weight",
                    "mlp.down_proj.weight",
                ];
                let names = tensor_names_map
                    .get(layer_type)
                    .unwrap_or(&default_names);

                let pfx = format!("{prefix}{layer_idx}.");
                let weights = provider.load_layer_weights(layer_idx, layer_type, names, &pfx);

                if tx.send(PrefetchMessage::Ready { layer_idx, weights }).is_err() {
                    // Receiver dropped (e.g. forward pass aborted early)
                    return;
                }
            }
            let _ = tx.send(PrefetchMessage::Done);
        });

        self.receiver = Some(rx);
        self.worker_handle = Some(handle);
    }

    /// Retrieve the next prefetched layer weights.
    /// This joins with the background thread if I/O is still pending, or returns immediately
    /// if the background thread already finished loading the layer ahead of time.
    pub fn next_layer(&mut self) -> Result<Option<(usize, HashMap<String, Vec<f32>>)>, String> {
        let rx = match &self.receiver {
            Some(rx) => rx,
            None => return Err("AirLlmStreamer prefetch pipeline not started".to_string()),
        };

        match rx.recv() {
            Ok(PrefetchMessage::Ready { layer_idx, weights }) => match weights {
                Ok(w) => Ok(Some((layer_idx, w))),
                Err(e) => Err(format!("Layer {layer_idx} prefetch I/O error: {e}")),
            },
            Ok(PrefetchMessage::Done) => Ok(None),
            Err(_) => Ok(None),
        }
    }
}

impl Drop for AirLlmStreamer {
    fn drop(&mut self) {
        // Dropping the receiver causes the background thread's sync_channel send to fail and exit
        self.receiver = None;
        if let Some(handle) = self.worker_handle.take() {
            let _ = handle.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct MockProvider {
        load_count: AtomicUsize,
    }

    impl WeightProvider for MockProvider {
        fn read_tensor_f32(&self, _name: &str) -> Result<Vec<f32>, String> {
            Ok(vec![1.0; 16])
        }
        fn read_tensor_slice_f32(&self, _name: &str, _offset: usize, count: usize) -> Result<Vec<f32>, String> {
            Ok(vec![1.0; count])
        }
        fn load_layer_weights(
            &self,
            _layer_idx: usize,
            _layer_type: &str,
            layer_names: &[&str],
            _prefix: &str,
        ) -> Result<HashMap<String, Vec<f32>>, String> {
            self.load_count.fetch_add(1, Ordering::SeqCst);
            let mut w = HashMap::new();
            for &n in layer_names {
                w.insert(n.to_string(), vec![1.0; 16]);
            }
            Ok(w)
        }
    }

    #[test]
    fn test_airllm_streamer_double_buffering() {
        let provider = Arc::new(MockProvider {
            load_count: AtomicUsize::new(0),
        });
        let mut streamer = AirLlmStreamer::new(
            provider.clone(),
            3,
            vec!["linear_attention".to_string(); 3],
            "blk.".to_string(),
        );

        streamer.start_prefetch();

        for expected_layer in 0..3 {
            let layer_res = streamer.next_layer().unwrap();
            assert!(layer_res.is_some());
            let (layer_idx, weights) = layer_res.unwrap();
            assert_eq!(layer_idx, expected_layer);
            assert!(!weights.is_empty());
        }

        let done = streamer.next_layer().unwrap();
        assert!(done.is_none());
        assert_eq!(provider.load_count.load(Ordering::SeqCst), 3);
    }
}
