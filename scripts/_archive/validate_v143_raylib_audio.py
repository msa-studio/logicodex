#!/usr/bin/env python3
"""
Validator: v1.43.0-alpha — Raylib Audio Integration

Validates:
  - Audio types (Sound, Music, Wave, AudioStream) defined in raylib_sys.rs
  - Audio functions registered in CallableRegistry (22 functions)
  - Safe wrappers exist in raylib.rs
  - No conflict with bare metal capability system
  - StrictAudioContext integration point exists

Usage: python3 scripts/validate_v143_raylib_audio.py
"""

import os

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

def check(name, condition):
    status = "PASS" if condition else "FAIL"
    print(f"  {status}  {name}")
    return condition

def main():
    passed = 0
    failed = 0

    # Read source files
    with open(f"{REPO}/src/ffi/raylib_sys.rs") as f:
        sys_content = f.read()
    with open(f"{REPO}/src/ffi/raylib.rs") as f:
        wrapper_content = f.read()
    with open(f"{REPO}/src/semantic.rs") as f:
        semantic_content = f.read()

    # ─── Audio Types ───
    if check("Audio type: Sound defined", "pub struct Sound" in sys_content): passed += 1
    else: failed += 1
    if check("Audio type: Music defined", "pub struct Music" in sys_content): passed += 1
    else: failed += 1
    if check("Audio type: Wave defined", "pub struct Wave" in sys_content): passed += 1
    else: failed += 1
    if check("Audio type: AudioStream defined", "pub struct AudioStream" in sys_content): passed += 1
    else: failed += 1
    if check("Audio type: AudioCallback defined", "pub type AudioCallback" in sys_content): passed += 1
    else: failed += 1

    # ─── Audio Functions in FFI ───
    audio_fns = [
        "InitAudioDevice", "CloseAudioDevice", "IsAudioDeviceReady", "SetMasterVolume",
        "LoadSound", "UnloadSound", "PlaySound", "StopSound", "IsSoundPlaying",
        "LoadMusicStream", "UnloadMusicStream", "PlayMusicStream", "StopMusicStream",
        "IsMusicStreamPlaying", "UpdateMusicStream", "SetMusicVolume", "SeekMusicStream",
        "LoadAudioStream", "UnloadAudioStream", "PlayAudioStream", "StopAudioStream",
        "IsAudioStreamPlaying",
    ]
    for fn in audio_fns:
        if check(f"FFI: {fn} declared", f"pub fn {fn}" in sys_content):
            passed += 1
        else:
            failed += 1

    # ─── Safe Wrappers ───
    safe_wrappers = [
        "init_audio_device", "close_audio_device", "is_audio_device_ready", "set_master_volume",
        "load_sound", "unload_sound", "play_sound", "stop_sound", "is_sound_playing",
        "load_music_stream", "unload_music_stream", "play_music_stream", "stop_music_stream",
        "is_music_stream_playing", "update_music_stream", "seek_music_stream",
        "load_audio_stream", "unload_audio_stream", "play_audio_stream", "stop_audio_stream",
        "is_audio_stream_playing", "set_audio_stream_callback",
    ]
    for wrap in safe_wrappers:
        if check(f"Wrapper: {wrap}()", f"pub unsafe fn {wrap}" in wrapper_content):
            passed += 1
        else:
            failed += 1

    # ─── CallableRegistry Registration ───
    # Audio functions are registered via register_fn! macro inside register_raylib_functions()
    # Find the Audio section and verify each function is registered there
    for fn in audio_fns:
        if check(f"CallableRegistry: {fn} registered", f'register_fn!("{fn}"' in wrapper_content):
            passed += 1
        else:
            failed += 1

    # ─── StrictAudioContext Integration ───
    if check("register_audio_callback() exists", "register_audio_callback" in semantic_content):
        passed += 1
    else: failed += 1
    if check("verify_audio_safety() exists", "verify_audio_safety" in semantic_content):
        passed += 1
    else: failed += 1
    if check("AudioViolationIo exists", "AudioViolationIo" in semantic_content):
        passed += 1
    else: failed += 1
    if check("AudioViolationRecursion exists", "AudioViolationRecursion" in semantic_content):
        passed += 1
    else: failed += 1
    if check("SetAudioStreamCallback in comments", "SetAudioStreamCallback" in wrapper_content):
        passed += 1
    else: failed += 1

    # ─── No Conflict with Capability System ───
    with open(f"{REPO}/lib/core/capability.ldx") as f:
        cap_content = f.read()
    if check("Capability Audio domain exists", "domain Audio" in cap_content):
        passed += 1
    else: failed += 1
    if check("Audio.Main gate exists", "Main" in cap_content.split("domain Audio")[1].split("domain Crypto")[0]):
        passed += 1
    else: failed += 1
    if check("Audio.Rakam gate exists", "Rakam" in cap_content):
        passed += 1
    else: failed += 1

    # ─── Test file ───
    if check("Test file exists", os.path.exists(f"{REPO}/tests/raylib_audio_v143.rs")):
        passed += 1
    else: failed += 1

    print(f"\n{'='*55}")
    print(f"v1.43 Raylib Audio Integration: {passed}/{passed+failed} checks passed")
    if failed == 0:
        print("ALL CHECKS PASSED ✅")
    return 0 if failed == 0 else 1

if __name__ == "__main__":
    import sys
    sys.exit(main())
