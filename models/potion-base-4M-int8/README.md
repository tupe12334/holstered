# potion-base-4M (int8)

The static-embedding model behind meaning recall (`src/recall/semantic.rs`),
compiled into the binary with `include_bytes!`.

- Source: [minishlab/potion-base-4M](https://huggingface.co/minishlab/potion-base-4M), MIT License, (c) The Minish Lab.
- Quantized to int8 (3.6 MB, from 14 MB float32) with
  `model2vec.StaticModel.from_pretrained("minishlab/potion-base-4M", quantize_to="int8").save_pretrained(...)`.
  On the eval it ranks the right skill first for all 32 labeled prompts, the same as float32.
