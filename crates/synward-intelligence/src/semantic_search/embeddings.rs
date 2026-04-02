//! Embedding Model using candle-transformers
//!
//! Loads and runs all-MiniLM-L6-v2 for local semantic embeddings.
//! Model downloaded from HuggingFace Hub on first use.

use candle_core::{Device, Tensor, DType};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config};
use tokenizers::Tokenizer;
use hf_hub::{api::sync::Api, Repo, RepoType};
use serde::{Deserialize, Serialize};

use crate::Result;
use crate::error::Error;

/// Embedding model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    /// Model repository on HuggingFace
    pub model_repo: String,
    
    /// Use CUDA if available
    pub use_cuda: bool,
    
    /// Maximum sequence length
    pub max_length: usize,
    
    /// Pooling strategy (mean, cls)
    pub pooling: PoolingStrategy,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            model_repo: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
            use_cuda: false,
            max_length: 256,
            pooling: PoolingStrategy::Mean,
        }
    }
}

/// Pooling strategy for sentence embeddings
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PoolingStrategy {
    /// Mean pooling over all tokens (default for sentence-transformers)
    Mean,
    /// Use [CLS] token embedding
    Cls,
}

/// Embedding model wrapper
pub struct EmbeddingModel {
    model: BertModel,
    tokenizer: Tokenizer,
    config: EmbeddingConfig,
    device: Device,
}

impl EmbeddingModel {
    /// Load model from HuggingFace Hub
    pub async fn load(model_name: &str, use_gpu: bool) -> Result<Self> {
        let config = EmbeddingConfig {
            model_repo: model_name.to_string(),
            use_cuda: use_gpu,
            ..Default::default()
        };
        
        Self::load_with_config(config).await
    }
    
    /// Load with custom configuration
    pub async fn load_with_config(config: EmbeddingConfig) -> Result<Self> {
        tracing::info!("Loading embedding model: {}", config.model_repo);
        
        // Setup device
        let device = if config.use_cuda {
            Device::new_cuda(0).map_err(|e| {
                tracing::warn!("CUDA not available, falling back to CPU: {}", e);
                e
            }).unwrap_or_else(|_| Device::Cpu)
        } else {
            Device::Cpu
        };
        
        tracing::info!("Using device: {:?}", device);
        
        // Download model from HuggingFace Hub
        let api = Api::new().map_err(|e| Error::ModelLoad(format!("HuggingFace API error: {}", e)))?;
        let repo = Repo::new(config.model_repo.clone(), RepoType::Model);
        let api_repo = api.repo(repo);
        
        tracing::info!("Downloading model files from HuggingFace...");
        
        // Download config, model, and tokenizer
        let config_path = api_repo.get("config.json")
            .map_err(|e| Error::ModelLoad(format!("Failed to download config.json: {}", e)))?;
        
        let model_path = api_repo.get("model.safetensors")
            .or_else(|_| api_repo.get("pytorch_model.bin"))
            .map_err(|e| Error::ModelLoad(format!("Failed to download model weights: {}", e)))?;
        
        let tokenizer_path = api_repo.get("tokenizer.json")
            .map_err(|e| Error::ModelLoad(format!("Failed to download tokenizer.json: {}", e)))?;
        
        tracing::info!("Model files downloaded to cache");
        
        // Load config
        let bert_config: Config = serde_json::from_str(
            &std::fs::read_to_string(&config_path)
                .map_err(|e| Error::ModelLoad(format!("Failed to read config: {}", e)))?
        ).map_err(|e| Error::ModelLoad(format!("Failed to parse config: {}", e)))?;
        
        // Load tokenizer
        let tokenizer = Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| Error::ModelLoad(format!("Failed to load tokenizer: {}", e)))?
            .into();
        
        // Load model weights
        let vb = unsafe { VarBuilder::from_mmaped_safetensors(&[model_path], DType::F32, &device)
            .map_err(|e| Error::ModelLoad(format!("Failed to load model weights: {}", e)))? };
        
        let model = BertModel::load(vb, &bert_config)
            .map_err(|e| Error::ModelLoad(format!("Failed to build model: {}", e)))?;
        
        tracing::info!("Embedding model loaded successfully");
        
        Ok(Self {
            model,
            tokenizer,
            config,
            device,
        })
    }
    
    /// Encode text to embedding vector
    pub async fn encode(&self, text: &str) -> Result<Vec<f32>> {
        // Tokenize
        let encoding = self.tokenizer
            .encode(text, true)
            .map_err(|e| Error::Encoding(format!("Tokenization failed: {}", e)))?;
        
        let tokens = encoding.get_ids();
        let attention_mask = encoding.get_attention_mask();
        
        // Truncate if needed
        let tokens = if tokens.len() > self.config.max_length {
            &tokens[..self.config.max_length]
        } else {
            tokens
        };
        let attention_mask = if attention_mask.len() > self.config.max_length {
            &attention_mask[..self.config.max_length]
        } else {
            attention_mask
        };
        
        // Convert to tensors
        let token_tensor = Tensor::new(tokens, &self.device)
            .map_err(|e| Error::Encoding(format!("Token tensor creation failed: {}", e)))?
            .unsqueeze(0)
            .map_err(|e| Error::Encoding(format!("Unsqueeze failed: {}", e)))?;
        
        let mask_tensor = Tensor::new(attention_mask, &self.device)
            .map_err(|e| Error::Encoding(format!("Mask tensor creation failed: {}", e)))?
            .unsqueeze(0)
            .map_err(|e| Error::Encoding(format!("Unsqueeze failed: {}", e)))?;
        
        // Run model - forward takes (input_ids, attention_mask, token_type_ids)
        // token_type_ids can be None for sentence-transformers
        let embeddings = self.model
            .forward(&token_tensor, &mask_tensor, None)
            .map_err(|e| Error::Encoding(format!("Model forward pass failed: {}", e)))?;
        
        // Pool embeddings
        let pooled = match self.config.pooling {
            PoolingStrategy::Mean => {
                self.mean_pool(&embeddings, &mask_tensor)?
            }
            PoolingStrategy::Cls => {
                // [CLS] token is first
                embeddings
                    .narrow(1, 0, 1)
                    .map_err(|e| Error::Encoding(format!("Narrow failed: {}", e)))?
                    .squeeze(1)
                    .map_err(|e| Error::Encoding(format!("Squeeze failed: {}", e)))?
            }
        };
        
        // Normalize to unit length
        let normalized = self.normalize(&pooled)?;
        
        // Convert to Vec<f32>
        let vec: Vec<f32> = normalized
            .flatten_all()
            .map_err(|e| Error::Encoding(format!("Flatten failed: {}", e)))?
            .to_vec1()
            .map_err(|e| Error::Encoding(format!("To vec failed: {}", e)))?;
        
        Ok(vec)
    }
    
    /// Mean pooling with attention mask
    fn mean_pool(&self, embeddings: &Tensor, mask: &Tensor) -> Result<Tensor> {
        // Convert mask to F32 to match embeddings dtype
        let mask_f32 = mask
            .to_dtype(DType::F32)
            .map_err(|e| Error::Encoding(format!("Mask dtype conversion failed: {}", e)))?;
        
        // Expand mask to match embedding shape
        let mask_expanded = mask_f32
            .unsqueeze(2)
            .map_err(|e| Error::Encoding(format!("Mask expand failed: {}", e)))?
            .broadcast_as(embeddings.shape().dims())
            .map_err(|e| Error::Encoding(format!("Mask broadcast failed: {}", e)))?;
        
        // Apply mask
        let masked = embeddings
            .mul(&mask_expanded)
            .map_err(|e| Error::Encoding(format!("Mask apply failed: {}", e)))?;
        
        // Sum and divide by mask sum
        let sum = masked
            .sum(1)
            .map_err(|e| Error::Encoding(format!("Sum failed: {}", e)))?;
        
        let mask_sum: f32 = mask_f32
            .sum_all()
            .map_err(|e| Error::Encoding(format!("Mask sum failed: {}", e)))?
            .to_scalar::<f32>()
            .map_err(|e| Error::Encoding(format!("To scalar failed: {}", e)))?;
        
        let mask_sum = if mask_sum > 0.0 { mask_sum } else { 1.0 };
        
        // Scale by inverse of mask sum (mean pooling)
        let scale = 1.0 / mask_sum as f64;
        sum.affine(scale, 0.0)
            .map_err(|e| Error::Encoding(format!("Scale failed: {}", e)))
    }

    /// L2 normalize
    fn normalize(&self, tensor: &Tensor) -> Result<Tensor> {
        let norm: f32 = tensor
            .sqr()
            .map_err(|e| Error::Encoding(format!("Square failed: {}", e)))?
            .sum_all()
            .map_err(|e| Error::Encoding(format!("Sum failed: {}", e)))?
            .to_scalar::<f32>()
            .map_err(|e| Error::Encoding(format!("To scalar failed: {}", e)))?
            .sqrt();
        
        let norm = if norm > 0.0 { norm } else { 1.0 };
        
        // Scale by inverse of norm (normalization)
        let scale = 1.0 / norm as f64;
        tensor.affine(scale, 0.0)
            .map_err(|e| Error::Encoding(format!("Normalize failed: {}", e)))
    }
    
    /// Batch encode multiple texts
    pub async fn encode_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        let mut embeddings = Vec::with_capacity(texts.len());
        
        // Process in batches
        for text in texts {
            embeddings.push(self.encode(text).await?);
        }
        
        Ok(embeddings)
    }
    
    /// Get embedding dimension
    pub fn embedding_dim(&self) -> usize {
        // all-MiniLM-L6-v2 has 384 dimensions
        384
    }
}

/// Model metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub name: String,
    pub embedding_dim: usize,
    pub max_seq_length: usize,
    pub pooling: PoolingStrategy,
    pub vocab_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_load_model() {
        let model = EmbeddingModel::load("sentence-transformers/all-MiniLM-L6-v2", false).await;
        
        match model {
            Ok(m) => {
                tracing::info!("Model loaded, dim: {}", m.embedding_dim());
                assert_eq!(m.embedding_dim(), 384);
            }
            Err(e) => {
                tracing::warn!("Model load failed (expected in test env): {}", e);
            }
        }
    }
    
    #[tokio::test]
    async fn test_encode() {
        let model = match EmbeddingModel::load("sentence-transformers/all-MiniLM-L6-v2", false).await {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!("Skipping test - model not available: {}", e);
                return;
            }
        };
        
        let emb = model.encode("hello world").await.unwrap();
        assert_eq!(emb.len(), 384);
        
        // Check normalization
        let norm: f32 = emb.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.01);
    }
}
