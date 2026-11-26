# Opening mission on M1 Max

Tested on an M1 Max with 64 GB, macOS 26.6.2 and CrossOver Preview using D3DMetal. The Series S run used the source-built executable with SHA-256 `5160347355c4e0e348b58dfbf53eab0c33434f1d46e3fd86fa610b02f01b9526`. Apply the two patches in this directory in order to [cool_fox f4b48ad](https://github.com/kitotake/cool_fox/tree/f4b48ad766ddbec1a1867022cd42f7645b0dd55e), starting with `display-and-rosetta.patch`.

This trace disabled deferred list recording and asynchronous submission preparation with `VICTORY_THIN=0` and `VICTORY_ASYNC_SUBMIT=0`. The ambient setting was `VICTORY_AMBIENT=0.8`. Diagnostics captured the submitted image and intermediate targets. Measurements with full target readback include its overhead.

| Frame | Result |
| --- | --- |
| 4273 | Depth and material targets show buildings, a palm tree and a street. The presented image was entirely black. |
| 4277 | The depth buffer also shows a character and vehicle. The 640×360 R11G11B10_FLOAT target written by compute shader `e6d8226a6870f577` contained a dim image of the street. The 1280×720 RGBA16_FLOAT color target sampled by the upscaler appeared black. Both 1920×1080 RGBA16_FLOAT targets written by `fdce2a0a9fb1521a` and `6a6c7b7fde59134f` also appeared black in readback. |
| 4291 | The captured back buffer showed a minimap and the BF GRETA HUD. Outside the HUD, all three channels were in the range 0–1 out of 255. |
| 4295 | The HUD was absent and the image returned to the same 0–1 range throughout. |

The player reported an entirely black window during the HUD check. A visible pause menu, response to camera input and sustained gameplay frame rate remain unverified. The data support tracing the 1280×720 color target through lighting and post-processing. An end-of-frame target snapshot cannot identify the first draw that lost the color or establish what a later pass sampled earlier in the same frame.

## Lighting and presentation follow-up

The HDR target at 1280×720 and the two RGBA16_FLOAT targets at 1920×1080 contained zero positive finite RGB values at the end of frames 4273 and 4277. At frame 4301, the 640×360 scene radiance cache had color in the sky and silhouetted geometry. The submitted image outside the HUD still had RGB values of 0 or 1. The original display was closed after this capture.

A separate run kept serial submission, selected `VICTORY_AMBIENT=constant:0.8` and disabled the full render-target dump. The ambient shader confirmed the constant on the GPU path. The sixth captured scene frame still showed RGB 0–1 outside the HUD.

An optional third patch, `source-radiance-experiment.patch`, adds `VICTORY_PRESENT_SCENE_COLOR=1`. It copies the title's shaded 640×360 scene radiance on the GPU when the title runs its radiance pass. After the producing list completes, the compositor can draw the recent color under the title's existing interface. The normal path is the default. This omits later title effects and the native upscaler.

The source-built executable had SHA-256 `7ffa46bd3309882bc8438fe8fa0b026f89afd29a2dc4261e49a5c9c5d2478b4b`. The compositor selected the completed image at frame 3657. Frames 3657 and 3658 contained faint distant lights. Frame 3662 remained too dark to navigate. Across six reporting windows with at least 100 draws per frame, median frame time was 7.58 seconds, including 6.77 seconds of copying and 12,247 MB copied per frame. The test was stopped under host disk pressure before player control could be verified. This experiment is unsuitable as a default graphics path.

## SDR output from the source-built compositor

The optional scene-color path now applies the piecewise sRGB encoding to the linear radiance sample before it blends the premultiplied HUD. The original game display planes and the HUD are already display-referred. Exposure is unity and highlights above SDR white clip. This experiment does not reconstruct the title’s tone mapper.

In the earlier serial trace, 52.15%, 62.11% and 94.49% of radiance pixels had positive finite RGB in frames 4273, 4277 and 4301 respectively. Their largest RGB channel values were 0.232421875, 3.125 and 33.0. A 1×1 float target read (10932.871, 13.416385, 13.416385, 0) in the first two frames and (64, 6, 6, 0) in the last. The engine parameter’s role has not been established. Intermediates saved by the diagnostic viewer are range-normalized and cube-root encoded. They cannot be compared visually to a raw SDR frame as a measure of exposure.

On the M1 Max D3DMetal device, the 8-bit grayscale ramp read back as 0, 6, 13, 34, 118, 188, 255 and 255 for linear inputs 0, 1/512, 1/256, 1/64, 0.18, 0.5, 1 and 4. A half-opaque red overlay over 0.5 gray yielded RGB (158, 94, 94). Opaque interface color yielded (64, 127, 191). The original scaled two-plane regression test also passed. The tested executable’s SHA-256 is `891412ce9d7aa8370a7a60034297635de0265ca1291b4fda54c52c5de52dce17`.

The corrected source build selected its completed radiance texture at frame 2760. The submitted image was black on that frame. Frame 2761 showed the “Vice City, Two Years Ago” title card. Frame 2765 showed lit windows on a dark street. Frame 2780 showed the silhouette of a vehicle against changing windows. In the 960×540 samples of the submitted image, 5,641 pixels (1.09%) and 7,498 pixels (1.45%) respectively reached 8/255 luminance in those last two frames. The earlier linear-output run had 148 such pixels at its sixth scene frame. These were separate executions with different camera and lighting states. The GPU ramp provides the controlled comparison.

Across the first 20 opening-scene reporting windows with at least 100 draws per frame, median frame time was 8.02 seconds (about 0.12 FPS), including 6.88 seconds preparing copies and 13,635 MB of copying per frame. The player reported being in the intro or that the scene was too dark to judge a camera turn. These measurements do not establish input-responsive gameplay.

## Visible world and controller check

A player-supplied screenshot at 03:25 PT shows the minimap and BF GRETA panel on the Mac display. A character and palm trees appear in silhouette against a white sky, with an almost entirely black foreground. The title bar reports 14,388.0 ms/frame and 3,063 draws/frame. The active source build logged the same pair at 11:25:04 UTC. It reported 12,759.9 ms preparing copies, 16,518.5 MB copied, and 1,379.6 ms on the GPU for that reporting window.

The player pressed the controller’s Menu button with the HUD showing and reported the world stayed visible with no pause menu. macOS listed a Bluetooth Xbox controller, and a headless `XInputGetState` check in the same CrossOver bottle returned success on slot 0 and `ERROR_DEVICE_NOT_CONNECTED` on slots 1–3. The emulator merges keyboard and physical pad input into one logical pad by default. Device presence does not establish that the title read the button press.

## Live controller and lighting-builder comparison

The same display run later submitted a skyline with visible windows on frame 2820, sixty frames after the first heavy scene frame. In the 960×540 image, 94.96% of the sky band (rows 24 through 134) was at luminance 255. Every pixel in the lower half had luminance below 8. With the game focused, the player reported no camera movement they could tie to a sustained right-stick hold and release. A separate headless XInput observer in the same bottle reported slot 0 connected, but never sampled right-stick X above 20,000 during its 120-second observation. This does not identify where the input was lost. The player subsequently closed this run.

Between 11:25:04 and 11:32:20 UTC, 46 renderer reports had a median of 9,900 buffer refresh entries per frame. The renderer classified zero entries as safe to skip in all 46 reports. Median time spent refreshing was 6.94 seconds per frame, with the submitting thread working while the GPU queue was empty for 6.89 seconds. Median estimated GPU occupancy in these reports was 13%.

The comparison enabled `VICTORY_BVH_BUILDER=1` on the same executable and copied the save directory before launch. All 53 of the title’s builder pipelines were available, and the renderer entered the builder path at 11:38:22 UTC. Frame 4307 was entirely black. This run then exited with an unhandled guest access violation before producing its next scheduled scene capture. The single heavy reporting window is insufficient to compare frame rates. The user’s original save directory was used again when restoring the previous graphics configuration.

## Aligned renderer copies under CrossOver

The restored scene showed lit windows and a vehicle silhouette at frame 6039. Frame 6079 showed legible intro credits over a white sky and a mostly black foreground. In the restored run's first 20 reporting windows with at least 100 draws per frame, the median frame took 7,864.75 ms, including 6,705.0 ms preparing copies. One 174 MiB mapped buffer accounted for a median of 23 whole-buffer copies per frame (4,196.4 MB), timed at 1,566.94 ms. The broader refresh counter also includes requests whose busy portions can be skipped. Its byte count should be interpreted as the length requested.

After the other three patches, apply `source-vector-copy.patch` and set `VICTORY_VECTOR_COPY=1` to test the aligned SSE2 path on Rosetta. The default remains the forward byte-copy path. This experiment keeps that path for short copies, unaligned addresses and forward overlap with the destination inside the source. Each aligned vector access stays within a 4 KiB page. It uses the renderer's existing fault handling. The source tree after all four patches is `2189f030780b43a461d73ed1cfbce4ef37e60221`. The release-optimized diagnostic executable (debug symbols disabled) has SHA-256 `ef82e8232637160502115c881732aa6783310a21a8f9611c9c8cf92ecde5d786`.

The same test executable passed the renderer's alignment, missing-page, sparse remap and two GPU compositor checks with the setting off and on. Two added tests cover forward overlap and tails, and the exact copied prefix at an inaccessible source or destination page. The full memory-helper suite stopped in an existing masked-merge test with a Rosetta instruction error while vector copying was disabled. That test exercises a separate assembly routine. The passing results above are for the individually selected tests.

The existing D3D12 shared-buffer benchmark measured the following microseconds per copy on M1 Max. Its timer reports the best of five passes at each size. The column printed as `rep movsb` in its original output calls the selected renderer routine in both cases.

| Bytes per copy | Vector option off | Vector option on |
| ---: | ---: | ---: |
| 256 | 0.16 | 0.02 |
| 4,096 | 1.86 | 0.40 |
| 65,536 | 25.04 | 3.31 |

### Opening-scene comparison with vector copies enabled

The optimized executable passed profile selection and presented loading artwork at frame 4096. The renderer started presenting the opening scene's radiance at frame 4797. The person at the Mac reported a dark world or silhouettes. This run saved the loading frame and reported in-world timings. Its five extra opening-scene image captures were disabled, so foreground appearance during the timed interval is a player observation. Player control is still unverified.

| Metric, median of the first 20 reports with at least 100 draws/frame | Restored baseline | Vector copies enabled |
| --- | ---: | ---: |
| Frame time | 7,864.75 ms | 5,014.70 ms |
| FPS at the median frame time | 0.127 | 0.199 |
| Time preparing copies per frame | 6,705.0 ms | 3,499.4 ms |
| Bytes copied per frame (decimal MB) | 15,354.45 | 15,154.05 |
| Whole-copy time for the 174 MiB buffer per frame | 1,566.94 ms | 198.14 ms |
| Whole copies of that buffer per frame | 23 | 22 |
| GPU time per frame | 901.00 ms | 737.65 ms |
| Draws per frame | 2,795.5 | 3,147 |

The baseline interval was 11:44:08–11:46:43 UTC and the optimized interval was 12:02:12–12:04:45 UTC. Each used the same M1 Max, CrossOver Preview, D3DMetal, Series S configuration and sRGB compositor. The optimized run started from a verified copy of the baseline profile. These are independent intro runs with different draw counts and cache state. The builds also differ in debug symbols and the extra capture setting. The measured 36.2% lower median frame time describes those two runs. The same-executable hardware fixture above isolates the copy change. The optimized game still takes about five seconds per frame and has incomplete lighting.
