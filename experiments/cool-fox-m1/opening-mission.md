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
