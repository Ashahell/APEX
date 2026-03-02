@echo off
set APEX_USE_LLM=1
set LLAMA_SERVER_URL=http://127.0.0.1:8080
set LLAMA_MODEL=F:/Projects/AMS2-Chief/RaceCoachAI/Models/Qwen2.5-14B-Instruct-Q4_K_M.gguf
cargo run --bin apex-router
