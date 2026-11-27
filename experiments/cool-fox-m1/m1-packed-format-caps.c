#define COBJMACROS
#include <windows.h>
#include <initguid.h>
#include <d3d12.h>
#include <stdio.h>

int main(void) {
    ID3D12Device *device = NULL;
    HRESULT result = D3D12CreateDevice(NULL, D3D_FEATURE_LEVEL_11_0, &IID_ID3D12Device, (void **)&device);
    if (FAILED(result)) {
        fprintf(stderr, "D3D12CreateDevice failed %#lx\n", (unsigned long)result);
        return 1;
    }
    const DXGI_FORMAT formats[] = {DXGI_FORMAT_B5G5R5A1_UNORM, DXGI_FORMAT_B4G4R4A4_UNORM, DXGI_FORMAT_R8G8B8A8_UNORM};
    puts("format,result,render_target,typed_view,typed_load,typed_store");
    for (size_t i = 0; i < sizeof(formats) / sizeof(formats[0]); ++i) {
        D3D12_FEATURE_DATA_FORMAT_SUPPORT support = {.Format = formats[i]};
        result = ID3D12Device_CheckFeatureSupport(device, D3D12_FEATURE_FORMAT_SUPPORT, &support, sizeof(support));
        printf("%d,%#lx,%d,%d,%d,%d\n", formats[i], (unsigned long)result,
               !!(support.Support1 & D3D12_FORMAT_SUPPORT1_RENDER_TARGET),
               !!(support.Support1 & D3D12_FORMAT_SUPPORT1_TYPED_UNORDERED_ACCESS_VIEW),
               !!(support.Support2 & D3D12_FORMAT_SUPPORT2_UAV_TYPED_LOAD),
               !!(support.Support2 & D3D12_FORMAT_SUPPORT2_UAV_TYPED_STORE));
        if (FAILED(result)) {
            ID3D12Device_Release(device);
            return 2;
        }
    }
    ID3D12Device_Release(device);
    return 0;
}
