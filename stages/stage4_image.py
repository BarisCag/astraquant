import os
import json
import gc
import torch
from diffusers import StableDiffusionPipeline

def clear_vram():
    """Strict VRAM clearing routine for 6GB RTX 2060."""
    print("Flushing VRAM and running garbage collection...")
    if torch.cuda.is_available():
        torch.cuda.empty_cache()
        torch.cuda.ipc_collect()
    gc.collect()

def run_image_generation(json_path="data/chapters.json", output_dir="data/images"):
    print("Initializing Stage 4: Local Diffusion Image Generation...")
    
    if not os.path.exists(json_path):
        print(f"Error: Schema file {json_path} not found. Ensure previous stages have run.")
        return
        
    with open(json_path, 'r', encoding='utf-8') as f:
        chapters = json.load(f)

    os.makedirs(output_dir, exist_ok=True)
    
    print("Loading Local Diffusion Model (optimizing for 6GB VRAM)...")
    model_id = "runwayml/stable-diffusion-v1-5"
    
    try:
        # Load in fp16 to save VRAM
        pipe = StableDiffusionPipeline.from_pretrained(
            model_id,
            torch_dtype=torch.float16,
            safety_checker=None
        )
        
        # Strict VRAM optimizations for RTX 2060
        pipe.enable_model_cpu_offload()
        pipe.enable_attention_slicing()
        pipe.enable_vae_slicing()
    except Exception as e:
        print(f"Failed to load diffusion pipeline: {e}")
        return

    for idx, chapter in enumerate(chapters):
        # Default prompt if schema doesn't provide one
        prompt = chapter.get("image_prompt", "A relaxing sleep environment, peaceful, dark, ambient, 4k resolution")
        print(f"Generating image for chapter {idx}...")
        
        try:
            # Generate image
            image = pipe(
                prompt,
                num_inference_steps=30,
                width=512,
                height=512
            ).images[0]
            
            out_path = os.path.join(output_dir, f"chapter_{idx:03d}.png")
            image.save(out_path)
            chapter["image_path"] = out_path
            print(f"Saved: {out_path}")
            
        except Exception as e:
            print(f"Error generating image for chapter {idx}: {e}")
            
        # STRICT VRAM CLEARING AFTER EVERY GENERATION
        clear_vram()
        
    # Save updated schema to coordinate with Stage 5
    updated_json_path = json_path.replace(".json", "_with_images.json")
    with open(updated_json_path, 'w', encoding='utf-8') as f:
        json.dump(chapters, f, indent=4)
    print(f"Updated schema saved to {updated_json_path}")
        
    # Final teardown
    del pipe
    clear_vram()
    print("Stage 4 complete. Images saved and VRAM cleared.")

if __name__ == "__main__":
    # Assumes run from project root ASTRA
    run_image_generation()
