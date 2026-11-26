//! Reserved GPU buffers over XMem physical heaps on Rosetta. A buffer's
//! GPU address survives the title remapping its CPU view.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock, Weak};

use windows::Win32::Graphics::Direct3D12::*;
use windows::core::Interface;

use crate::alias::PhysicalHeap;
use crate::watch::PAGE;

/// A span of consecutive tiles in an XMem view and its physical heap.
pub struct PhysicalTiles {
    pub first: u32,
    pub count: u32,
    pub heap_tile: u32,
    pub heap: Arc<PhysicalHeap>,
}

type Layout = fn(usize, usize) -> Option<Vec<PhysicalTiles>>;
static LAYOUT: OnceLock<Layout> = OnceLock::new();
static CHANGED: AtomicBool = AtomicBool::new(false);
static QUEUE: AtomicUsize = AtomicUsize::new(0);
static BUFFERS: Mutex<Vec<Weak<ReservedBuffer>>> = Mutex::new(Vec::new());

/// Sets the runtime callback that snapshots mappings of an XMem buffer.
pub fn use_physical_layout(layout: Layout) {
    if crate::alias::physical_heaps_enabled() {
        let _ = LAYOUT.set(layout);
    }
}

/// The runtime completed an XMem remap. The next host submission uses it.
pub fn physical_layout_changed() {
    CHANGED.store(true, Ordering::Release);
}

pub(crate) struct ReservedBuffer {
    pub(crate) resource: ID3D12Resource,
    base: usize,
    len: usize,
    count: u32,
    mapped: Mutex<Vec<PhysicalTiles>>,
}

impl ReservedBuffer {
    pub(crate) fn new(base: usize, len: usize) -> Option<Arc<Self>> {
        if len == 0 || !base.is_multiple_of(PAGE) || !len.is_multiple_of(PAGE) {
            return None;
        }
        let count = u32::try_from(len / PAGE).ok()?;
        let layout = LAYOUT.get()?(base, len)?;
        if layout.iter().map(|run| run.count).sum::<u32>() != count {
            return None;
        }
        let host = crate::host::host()?;
        let desc =
            super::alias::physical_buffer_desc(len, D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS);
        let mut resource: Option<ID3D12Resource> = None;
        // SAFETY: ordinary resource creation with a sized output slot.
        unsafe {
            host.device.CreateReservedResource(
                &desc,
                D3D12_RESOURCE_STATE_COMMON,
                None,
                &mut resource,
            )
        }
        .ok()?;
        let buffer = Arc::new(Self {
            resource: resource?,
            base,
            len,
            count,
            mapped: Mutex::new(Vec::new()),
        });
        BUFFERS
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(Arc::downgrade(&buffer));
        physical_layout_changed();
        if len >= 16 << 20 {
            tracing::info!(
                base = format_args!("{base:#x}"),
                len,
                "title buffer uses shared tiles"
            );
        }
        Some(buffer)
    }

    /// Under the host submission lock. Keeps the old heaps until mapping
    /// updates and the lists that follow them have completed.
    fn bind(&self, queue: &ID3D12CommandQueue, new_queue: bool) -> Vec<PhysicalTiles> {
        let layout = LAYOUT
            .get()
            .and_then(|layout| layout(self.base, self.len))
            .unwrap_or_else(|| {
                crate::host::fail(format_args!(
                    "XMem lost the layout for GPU buffer {:#x}",
                    self.base
                ));
            });
        let mut mapped = self.mapped.lock().unwrap_or_else(|e| e.into_inner());
        if !new_queue
            && mapped.len() == layout.len()
            && mapped.iter().zip(&layout).all(|(a, b)| {
                a.first == b.first
                    && a.count == b.count
                    && a.heap_tile == b.heap_tile
                    && Arc::ptr_eq(&a.heap, &b.heap)
            })
        {
            return Vec::new();
        }
        let start = D3D12_TILED_RESOURCE_COORDINATE::default();
        let size = D3D12_TILE_REGION_SIZE {
            NumTiles: self.count,
            ..Default::default()
        };
        // SAFETY: the coordinate and region cover exactly this buffer.
        unsafe {
            queue.UpdateTileMappings(
                &self.resource,
                1,
                Some(&start),
                Some(&size),
                None::<&ID3D12Heap>,
                1,
                Some(&D3D12_TILE_RANGE_FLAG_NULL),
                None,
                Some(&self.count),
                D3D12_TILE_MAPPING_FLAG_NONE,
            );
        }
        for run in &layout {
            let start = D3D12_TILED_RESOURCE_COORDINATE {
                X: run.first,
                ..Default::default()
            };
            let size = D3D12_TILE_REGION_SIZE {
                NumTiles: run.count,
                ..Default::default()
            };
            // SAFETY: the runtime cut each run to this buffer and its host
            // heap. The new and old heap owners stay alive until the fence.
            unsafe {
                queue.UpdateTileMappings(
                    &self.resource,
                    1,
                    Some(&start),
                    Some(&size),
                    &run.heap.heap,
                    1,
                    Some(&D3D12_TILE_RANGE_FLAG_NONE),
                    Some(&run.heap_tile),
                    Some(&run.count),
                    D3D12_TILE_MAPPING_FLAG_NONE,
                );
            }
        }
        std::mem::replace(&mut *mapped, layout)
    }
}

pub(crate) struct InFlightTiles {
    _buffers: Vec<Arc<ReservedBuffer>>,
    _prior: Vec<PhysicalTiles>,
}

/// Updates mappings on the very queue that is about to use the buffers.
pub(crate) fn prepare(queue: &ID3D12CommandQueue) -> Option<InFlightTiles> {
    LAYOUT.get()?;
    let address = queue.as_raw() as usize;
    let new_queue = QUEUE.swap(address, Ordering::Relaxed) != address;
    if !CHANGED.swap(false, Ordering::AcqRel) && !new_queue {
        return None;
    }
    let buffers = {
        let mut registry = BUFFERS.lock().unwrap_or_else(|e| e.into_inner());
        let mut live = Vec::with_capacity(registry.len());
        registry.retain(|buffer| match buffer.upgrade() {
            Some(buffer) => {
                live.push(buffer);
                true
            }
            None => false,
        });
        live
    };
    let prior = buffers
        .iter()
        .flat_map(|buffer| buffer.bind(queue, new_queue))
        .collect();
    Some(InFlightTiles {
        _buffers: buffers,
        _prior: prior,
    })
}
