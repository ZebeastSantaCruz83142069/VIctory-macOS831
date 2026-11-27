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


## Locating the loss of HDR scene color

The opt-in `VICTORY_TRACE_WORLD_COLOR=1` diagnostic copies the native 1280×720 RGBA16_FLOAT target immediately before and after each selected dispatch on its producing command list. It restores the incoming resource state, waits for submission completion before reading, and excludes row padding from the BLAKE3 hash. It samples the first, sixth and twenty-first occurrences. The executable was built from the five patches on this branch. The resulting source tree is `5add3a416733a44bbd45b5c17e7c2354e501d234` and the executable SHA-256 is `d15c99847d9ed15e0757220ca5653d0512c6a02b5ad776c4695e41baffae37a1`. The D3DMetal fixture changed three known pixels in the lower half of a six-pixel FP16 texture. Both readbacks counted exactly the expected pixels and had distinct hashes. The sparse-copy and two compositor tests also passed.

The M1 Max test used a verified copy of the optimized save, Series S, serialized submission, aligned vector copies and the experimental radiance presenter. The table counts pixels with at least one positive finite RGB channel. Each full frame has 921,600 pixels and each lower half has 460,800. No sample contained nonfinite RGB channels.

| Frame | Occurrence | Compute pass | Whole image, before → after | Lower half, before → after |
| ---: | ---: | --- | ---: | ---: |
| 2974 | 1 | tiled lighting | 0 → 0 | 0 → 0 |
| 2974 | 1 | sky | 5,920 → 772,357 | 482 → 422,298 |
| 2974 | 1 | scattering | 772,357 → 0 | 422,298 → 0 |
| 2978 | 6 | tiled lighting | 0 → 0 | 0 → 0 |
| 2978 | 6 | sky | 17,045 → 622,689 | 0 → 247,457 |
| 2978 | 6 | scattering | 622,689 → 0 | 247,457 → 0 |
| 2993 | 21 | tiled lighting | 0 → 0 | 0 → 0 |
| 2993 | 21 | sky | 19,561 → 279,731 | 16,455 → 147,027 |
| 2993 | 21 | scattering | 279,731 → 0 | 147,027 → 0 |

On all three frames the sky pass populated the native HDR target. Immediately before atmospheric scattering, hundreds of thousands of pixels carried positive RGB. Immediately after, every byte of all four channels matched an independently generated zero texture. The tiled lighting dispatch also entered and left this target at zero. Its conditional zero store is consistent with this observation. The sky and scattering hashes were taken independently around each dispatch. Other work may occur between the two.

The captured display still showed a dark silhouette with a few lights on frame 2994. It was presenting the separate radiance texture. This test establishes the location of a complete loss of scene color in the title's HDR path. It leaves open whether the atmosphere shader loads the wrong value or receives invalid fog inputs. A previous frame capture showed the 480×272 R10G10B10A2 input at table 1 offset 24 as black in the exported RGB image. It lists two indirect compute writers for that input, both with successfully created host pipelines. The capture did not measure their output alpha or the number of workgroups each actually ran.


## Normal display with atmospheric scattering excluded

A second run used the exact tested binary and a verified copy of the save, with the native SDR planes selected. It set `VICTORY_PRESENT_SCENE_COLOR=0` and `VICTORY_SKIP_PIPELINES=577248728544d74b`. The normal post-processing and two-plane presentation continued. The log identified precisely this skipped shader. The overlay and base were both 1920×1080 RGBA8_UNORM. The submitted image was sampled at 960×540. Because the previous run used the early radiance presenter, this comparison changes both the atmosphere pass and the selected presentation source.

The sky pass left 185,191, 638,085 and 918,065 out of 921,600 pixels with positive RGB at its first, sixth and twenty-first occurrences. The lower halves held 113,188, 320,005 and 460,372 positive pixels. On the twenty-first occurrence (frame 3198), 917,042 pixels were already positive as the earlier tiled-lighting pass began. Color therefore survived into work on a later frame.

| Submitted frame | Largest RGB byte | Pixels at or above 8/255 luminance | Lower-half pixels at that luminance |
| ---: | ---: | ---: | ---: |
| 3179 | 1 | 0 | 0 |
| 3180 | 1 | 0 | 0 |
| 3184 | 1 | 0 | 0 |
| 3199 | 1 | 0 | 0 |

A separate synthetic compute test on the same M1 Max and D3DMetal queried RGBA16_FLOAT typed UAV load/store support and exercised a 16×16 texture using two DXC-compiled shaders. The first wrote known RGBA values, then a UAV barrier preceded an in-place componentwise multiply-add. All 1,024 FP16 channel readbacks matched exactly. This confirms the basic typed read/write operation on this device. It leaves the translated atmosphere shader, bindings, inputs and downstream passes to be distinguished.


## Presenting the native HDR texture before atmospheric scattering

The additional `source-pre-scattering.patch` introduces `VICTORY_PRESENT_SCENE_COLOR=before-scattering`. It snapshots the original 1280×720 RGBA16_FLOAT scene immediately before the atmosphere dispatch. The title executes that dispatch and the rest of its frame normally. For display, the presenter reads the completed snapshot in its native format, applies the existing linear-to-sRGB conversion and blends the title’s SDR HUD. The snapshot path omits the later atmospheric, exposure, upscaling and other post-processing effects from the displayed world. The original display-plane setting and the earlier radiance option are still available. All six patches applied in order reconstruct the source tree `fd24fbcd02dcb7205c225be2b76c048dee5a59ab`. The tested executable's SHA-256 is `a200e679d3c040addc1c0114113630528c8f04df37c17386984c8059dfb99f33`.

The hardware fixture overwrote the original FP16 texture after taking the snapshot. The saved snapshot retained all four channels byte-for-byte, and the original read back black. Both the FP16 and packed-float scene formats passed the SDR ramp and HUD composition test on the M1 Max. The sparse-copy fixture passed. A fifth sampled image from the preceding normal-plane run, frame 3239, also contained only levels 0 and 1 in every RGB channel.

The first full-resolution launch exited during title startup before recording a scene. A new, verified copy of the last save that reached the world was used to retry the same executable. This run retained Series S, vector copies and serialized submission. The atmosphere dispatch was enabled. The 960×540 samples below are readbacks from the submitted image. The lower half contains 259,200 pixels and the full image contains 518,400.

| Frame | Offset from first heavy frame | Pixels at or above 8/255 luminance | Lower half at or above 8/255 | Pixels at or above 248/255 |
| ---: | ---: | ---: | ---: | ---: |
| 3239 | 0 | 0 | 0 | 0 |
| 3240 | 1 | 518,400 | 259,200 | 0 |
| 3244 | 5 | 3,723 | 167 | 0 |
| 3259 | 20 | 9,757 | 414 | 36 |
| 3299 | 60 | 260,946 | 29,999 | 36,397 |

Frame 3239 was black and frame 3240 showed the dark loading background. Frame 3244 showed lit building windows on a dark street. Frame 3259 showed additional buildings with distinct purple, yellow and green lights. Most of the street foreground was still black. In frame 3259 only 414 of 259,200 lower-half pixels (0.16%) reached 8/255 luminance.

Across the first 20 reports with at least 100 draws per frame, median frame time was 6,066.75 ms (0.165 FPS), GPU time was 865.95 ms, and copy preparation took 4,661.4 ms for 15,569.1 MB per frame. These are timings from the diagnostic run. The image now shows a city before atmosphere, with incomplete foreground lighting. Player control and real-time gameplay remain unverified.

At frame 3299, sixty presented frames into the scene, the window showed a blue-purple sky, a line of trees and legible pink opening credits. The ground away from the credits remained almost black. In the lower-right 640×270 area of the 960×540 readback, 660 of 172,800 pixels reached 8/255 luminance. The person at the Mac reported still being in the intro or that it was too dark to judge the opposite camera inputs. The full-resolution run was left open.

## Sparse refreshes and foreground exposure on M1 Max

The source-built Series S variant used CrossOver Preview with D3DMetal on an Apple M1 Max. The optional `source-sparse-exposure.patch` follows `source-pre-scattering.patch`. The tested executable SHA-256 was `43d300ecc914f5ae9d70dcad1da5c13bcce829a3431ac8c0a7749389d9de2566`.

A repeated sparse upload previously copied a committed prefix, encountered a gap and copied the prefix again while walking memory regions. With `VICTORY_SPARSE_HINTS=1` a thread remembers up to 64 exact ranges and walks current regions first for a repeat. It retains neither the bytes nor the memory protections. A CrossOver test checked readable islands, untouched output boundaries, newly written edges and a formerly absent block that became committed. Alternating trials on a 16 MiB buffer with a 64 KiB gap near the end gave medians of 1,642 and 1,393 µs with hints off and 753 and 687 µs with hints on (21 copies per trial).

The following medians cover the first 20 renderer reports with at least 100 draws/frame. Each run started from a separate copy of the same saved profile. The opening shot, shader-cache state and competing Mac activity varied. The exposure run included the extra bounded upscaler readbacks. These measurements compare the overall runs and do not isolate an in-game speedup attributable to the sparse-copy change.

| Run | Frame time (ms) | Preparing copies (ms) | Copy volume (MB) | GPU (ms) | Draws/frame |
| --- | ---: | ---: | ---: | ---: | ---: |
| Previous HDR preview | 6,066.8 | 4,661.4 | 15,569.1 | 866.0 | 3,125.5 |
| Hints on, HDR preview +4 EV | 7,498.0 | 4,816.9 | 15,258.5 | 1,541.8 | 3,002.5 |

The HDR preview multiplies the scene's linear RGB by 16 before SDR encoding. Tests on the physical GPU checked a dim red surface, exact black and an opaque HUD pixel. The game still displayed black street geometry below the horizon. At scene +5 and +20, 898 and 609 of the 259,200 lower-half pixels met the 8/255 luminance threshold. Frame 3302 shows illuminated building windows around a silhouetted character. Raising exposure did not recover the missing light.

At the first scattering sample, 113170 pixels in the lower half of the native 1280×720 HDR surface had positive RGB. Both 1920×1080 RGBA16F upscaler targets (`6a6c7b7fde59134f`, `fdce2a0a9fb1521a`) had zero positive RGB after their dispatch at that occurrence. This run kept the atmosphere pass enabled. Player control and sustained gameplay at 60 FPS remain unverified.

## Color at the game upscaler on M1 Max

The Series S trial used source commit `5ce9f06`, CrossOver Preview and D3DMetal on the M1 Max. Executable SHA-256 was `fd5d87ccee93f7d8bce673a81730e2c738c0968cef7a7c2909f697b9a1d911ed`. A GPU test checked an SRV-state snapshot against the original texture after a clear. The game run used a copied save, excluded the scattering dispatch `577248728544d74b`, and selected the existing constant indirect-light estimate at 0.8. It also disabled the memory-map census and the experimental sparse refresh hint.

| Pass hash prefix | Frame | Readback | Positive RGB pixels | Positive RGB pixels in lower half |
| --- | ---: | --- | ---: | ---: |
| `6a6c` | 2669 | input | 180,808 | 112,979 |
| `6a6c` | 2669 | after | 431,587 | 262,314 |
| `fdce` | 2669 | input | 181,855 | 113,119 |
| `fdce` | 2669 | after | 450,575 | 269,364 |
| `6a6c` | 2673 | input | 804,249 | 360,178 |
| `6a6c` | 2673 | after | 1,827,405 | 822,887 |
| `fdce` | 2674 | input | 807,056 | 362,084 |
| `fdce` | 2674 | after | 1,869,493 | 849,643 |
| `6a6c` | 2688 | input | 921,600 | 460,800 |
| `6a6c` | 2688 | after | 2,073,600 | 1,036,800 |
| `fdce` | 2689 | input | 921,600 | 460,800 |
| `fdce` | 2689 | after | 2,073,600 | 1,036,800 |

The input is 1280×720 RGBA16F. Each output is 1920×1080 RGBA16F. On frame 2689 the second output had 76,052 lower-half pixels with at least one channel at or above 0.25. The input for that same dispatch had 1,925. These are samples of the individual surfaces, which have different resolutions and may have different encodings.

Presenting the upscaler input in the game window still left the foreground mostly black. At +5 and +20 from the first busy frame, 222 and 2,012 of 259,200 lower-half SDR pixels reached 8/255 luminance. The two measured atmosphere-bypass runs used different display and ambient settings. The earlier normal-display run stayed black. The new readbacks establish that the dispatches can produce color and leave open what the later post-processing and display stages do with it. The first 20 reports at 100 or more draws had medians of 4,628.45 ms/frame and 3,458.3 ms in copy preparation. Changed settings and host activity prevent attributing those figures to a single change.

## Upstream daylight fix and Series S startup on M1 Max

I built upstream [`0c0d5ca7`](https://github.com/pragyaangaur/VIctory-macOS/commit/0c0d5ca7f990fa4dee5c088933f2d184fb2229aa) from source at the exact published tree. It includes the packed-format daylight fix in [#478](https://github.com/pragyaangaur/VIctory-macOS/pull/478). The Windows build SHA-256 was `f9f46871c7330eaf9310631c383f4b04a67eab09f29db98e2af2e451f7fbd142`. CrossOver Preview and D3DMetal on the Apple M1 Max reported mesh shaders and Shader Model 6.6.

The included generated D3D12 capability query ran against the same device. All three format queries succeeded.

| DXGI format | Render target | Typed UAV view | Typed UAV load | Typed UAV store |
| --- | --- | --- | --- | --- |
| B5G5R5A1_UNORM (86) | Yes | No | Yes | No |
| B4G4R4A4_UNORM (115) | Yes | No | Yes | No |
| R8G8B8A8_UNORM (28) | Yes | Yes | Yes | Yes |

Series S reported 1280×720 and 3840×2160 presentation planes. On separate copies of the same saved profile, both planes stayed black in the observed startup samples. The readbacks are saved at half size.

| Settings | Selected plane | Last sampled black frame | Readback size |
| --- | ---: | ---: | --- |
| Largest plane, compositing on | 1 | 4096 | 1920×1080 |
| Plane 0, compositing off | 0 | 2048 | 640×360 |
| Plane 0, compositing and aliasing off | 0 | 2048 | 640×360 |

The title presented continuously with about one draw per frame. The live key file accepted Enter in the no-aliasing run. There was no opening world frame in these samples, so they do not exercise the sunlight fix in game or measure gameplay performance.
