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
