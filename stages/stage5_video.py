import os
import json
import subprocess

def get_audio_duration(audio_path):
    """Retrieve duration of audio file using ffprobe."""
    cmd = [
        "ffprobe", "-v", "error", "-show_entries",
        "format=duration", "-of", "default=noprint_wrappers=1:nokey=1",
        audio_path
    ]
    try:
        result = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, check=True)
        return float(result.stdout.strip())
    except Exception as e:
        print(f"Error reading duration for {audio_path}: {e}")
        return 0.0

def run_video_assembly(json_path="data/chapters_with_images.json", output_dir="data/video", temp_dir="data/temp"):
    print("Initializing Stage 5: FFmpeg Ken Burns & Crossfade Automation...")
    
    if not os.path.exists(json_path):
        print(f"Error: Schema file {json_path} not found. Ensure Stage 4 was run.")
        return
        
    with open(json_path, 'r', encoding='utf-8') as f:
        chapters = json.load(f)

    os.makedirs(output_dir, exist_ok=True)
    os.makedirs(temp_dir, exist_ok=True)
    
    intermediate_clips = []
    fps = 30
    fade_duration = 2.0
    
    print("Step 1: Generating intermediate Ken Burns clips for each chapter...")
    for idx, chapter in enumerate(chapters):
        image_path = chapter.get("image_path")
        audio_path = chapter.get("audio_path")
        
        if not image_path or not os.path.exists(image_path):
            print(f"Warning: Missing image for chapter {idx}, skipping...")
            continue
        if not audio_path or not os.path.exists(audio_path):
            print(f"Warning: Missing audio for chapter {idx}, skipping...")
            continue
            
        duration = get_audio_duration(audio_path)
        if duration <= fade_duration:
            print(f"Warning: Audio duration ({duration}s) too short for fade ({fade_duration}s) in chapter {idx}")
            continue
            
        out_clip = os.path.join(temp_dir, f"clip_{idx:03d}.mp4")
        frames = int(duration * fps)
        
        # Ken Burns effect: Scale up first to retain quality during zoompan, then zoom slowly to center.
        vf_filter = (
            f"scale=3840:-1,"
            f"zoompan=z='min(zoom+0.0005,1.5)':d={frames}:x='iw/2-(iw/zoom/2)':y='ih/2-(ih/zoom/2)':s=1920x1080,"
            f"framerate={fps}"
        )
        
        cmd = [
            "ffmpeg", "-y",
            "-loop", "1", "-i", image_path,
            "-i", audio_path,
            "-vf", vf_filter,
            "-c:v", "libx264", "-pix_fmt", "yuv420p",
            "-c:a", "aac", "-b:a", "192k",
            "-shortest",
            out_clip
        ]
        
        print(f"Processing chapter {idx} video (Duration: {duration:.2f}s)...")
        subprocess.run(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        
        if os.path.exists(out_clip):
            intermediate_clips.append((out_clip, duration))
        
    if not intermediate_clips:
        print("No valid clips were generated. Exiting Stage 5.")
        return
        
    print("Step 2: Crossfading clips together into final assembly...")
    final_output = os.path.join(output_dir, "final_sleep_video.mp4")
    
    if len(intermediate_clips) == 1:
        # Only one clip, simply copy to final output
        os.rename(intermediate_clips[0][0], final_output)
        print(f"Final video saved to {final_output}")
        return

    filter_script_path = os.path.join(temp_dir, "filter.txt")
    
    # Build filtergraph script for Xfade
    # Formula: next offset = current offset + next clip duration - fade_duration
    with open(filter_script_path, 'w') as f:
        current_offset = intermediate_clips[0][1] - fade_duration
        
        # Initial fade between clip 0 and 1
        f.write(f"[0:v][1:v]xfade=transition=fade:duration={fade_duration}:offset={current_offset}[v1];\n")
        f.write(f"[0:a][1:a]acrossfade=d={fade_duration}[a1];\n")
        
        last_v = "v1"
        last_a = "a1"
        
        # Subsequent fades
        for i in range(2, len(intermediate_clips)):
            current_offset += intermediate_clips[i-1][1] - fade_duration
            next_v = f"v{i}"
            next_a = f"a{i}"
            f.write(f"[{last_v}][{i}:v]xfade=transition=fade:duration={fade_duration}:offset={current_offset}[{next_v}];\n")
            f.write(f"[{last_a}][{i}:a]acrossfade=d={fade_duration}[{next_a}];\n")
            last_v = next_v
            last_a = next_a
            
    # Final concatenation execution
    cmd = ["ffmpeg", "-y"]
    for clip, _ in intermediate_clips:
        cmd.extend(["-i", clip])
        
    cmd.extend([
        "-filter_complex_script", filter_script_path,
        "-map", f"[{last_v}]",
        "-map", f"[{last_a}]",
        "-c:v", "libx264", "-pix_fmt", "yuv420p",
        "-c:a", "aac", "-b:a", "192k",
        final_output
    ])
    
    print("Executing final FFmpeg assembly. This may take some time depending on video length...")
    subprocess.run(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    
    if os.path.exists(final_output):
        print(f"Stage 5 complete! Final crossfaded video rendered successfully at: {final_output}")
    else:
        print("Error: Final video assembly failed. Please check the FFmpeg logs.")

if __name__ == "__main__":
    run_video_assembly()
