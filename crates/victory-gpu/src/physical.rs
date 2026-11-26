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

/// Current physical mappings. A reservation ID means the runtime owns
/// the whole address range, including the gaps between mappings.
pub struct PhysicalLayout {
    pub tiles: Vec<PhysicalTiles>,
    pub reservation: Option<u64>,
}

type Layout = fn(usize, usize) -> Option<PhysicalLayout>;
static LAYOUT: OnceLock<Layout> = OnceLock::new();
static CHANGED: AtomicBool = AtomicBool::new(false);
static QUEUE: AtomicUsize = AtomicUsize::new(0);
static BUFFERS: Mutex<Vec<Weak<ReservedBuffer>>> = Mutex::new(Vec::new());

/// An ExecuteCommandLists call can contain several lists. Keep its
/// reserved buffers physically disjoint so none needs a barrier between
/// accesses to two different resources inside that call.
fn overlapping_owners<'a>(layouts: impl IntoIterator<Item = &'a [PhysicalTiles]>) -> bool {
    let mut spans: Vec<_> = layouts
        .into_iter()
        .enumerate()
        .flat_map(|(owner, runs)| {
            runs.iter().map(move |run| {
                (
                    Arc::as_ptr(&run.heap) as usize,
                    run.heap_tile,
                    run.heap_tile + run.count,
                    owner,
                )
            })
        })
        .collect();
    spans.sort_unstable();
    let mut rightmost: Option<(usize, u32, usize)> = None;
    for (heap, from, to, owner) in spans {
        if let Some((last_heap, last_end, last_owner)) = rightmost
            && heap == last_heap
            && from < last_end
        {
            if owner != last_owner {
                return true;
            }
            if to <= last_end {
                continue;
            }
        }
        rightmost = Some((heap, to, owner));
    }
    false
}

/// Sets the runtime callback that snapshots mappings of an XMem buffer.
pub fn use_physical_layout(layout: Layout) {
    if crate::alias::physical_heaps_enabled() {
        let _ = LAYOUT.set(layout);
    }
}

/// Lets a title's XMem reservation start with holes in its GPU mapping.
pub fn sparse_physical_buffers() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        crate::switch::on("VICTORY_PHYSICAL_SPARSE") && crate::alias::physical_heaps_requested()
    })
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
    reservation: Option<u64>,
    guard: PhysicalHeap,
    mapped: Mutex<Vec<PhysicalTiles>>,
}

impl ReservedBuffer {
    pub(crate) fn new(base: usize, len: usize) -> Option<Arc<Self>> {
        if len == 0 || !base.is_multiple_of(PAGE) || !len.is_multiple_of(PAGE) {
            return None;
        }
        let count = u32::try_from(len / PAGE).ok()?;
        let snapshot = LAYOUT.get()?;
        let layout = snapshot(base, len)?;
        let mapped_tiles = layout.tiles.iter().map(|run| run.count).sum::<u32>();
        if layout.reservation.is_none() && mapped_tiles != count {
            return None;
        }
        let mut registry = BUFFERS.lock().unwrap_or_else(|e| e.into_inner());
        let others = registry
            .iter()
            .filter_map(Weak::upgrade)
            .map(|other| other.current_layout(*snapshot))
            .collect::<Option<Vec<_>>>()?;
        if overlapping_owners(
            std::iter::once(layout.tiles.as_slice()).chain(others.iter().map(Vec::as_slice)),
        ) {
            return None;
        }
        let host = crate::host::host()?;
        let desc = super::alias::physical_buffer_desc(
            len.checked_add(PAGE)?,
            D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS,
        );
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
            reservation: layout.reservation,
            guard: PhysicalHeap::new(&host.device, PAGE).ok()?,
            mapped: Mutex::new(Vec::new()),
        });
        registry.push(Arc::downgrade(&buffer));
        physical_layout_changed();
        if len >= 16 << 20 {
            tracing::info!(
                base = format_args!("{base:#x}"),
                len,
                mapped_bytes = mapped_tiles as usize * PAGE,
                "title buffer uses shared tiles"
            );
        }
        Some(buffer)
    }

    fn current_layout(&self, snapshot: Layout) -> Option<Vec<PhysicalTiles>> {
        let layout = snapshot(self.base, self.len)?;
        if self.reservation.is_some() && self.reservation != layout.reservation {
            // The title released the original placeholder. A new
            // reservation at the same CPU address is unrelated.
            return Some(Vec::new());
        }
        Some(layout.tiles)
    }

    /// Under the host submission lock. Keeps the old heaps until mapping
    /// updates and the lists that follow them have completed.
    fn bind(
        &self,
        queue: &ID3D12CommandQueue,
        layout: Vec<PhysicalTiles>,
        new_queue: bool,
    ) -> Vec<PhysicalTiles> {
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
        if layout.is_empty() {
            // D3DMetal can retain stale mappings when a buffer loses its
            // last tile. Bind the extra tile outside the title-visible range.
            let guard = D3D12_TILED_RESOURCE_COORDINATE {
                X: self.count,
                ..Default::default()
            };
            let one = D3D12_TILE_REGION_SIZE {
                NumTiles: 1,
                ..Default::default()
            };
            // SAFETY: this resource has one extra tile and owns its heap.
            unsafe {
                queue.UpdateTileMappings(
                    &self.resource,
                    1,
                    Some(&guard),
                    Some(&one),
                    &self.guard.heap,
                    1,
                    Some(&D3D12_TILE_RANGE_FLAG_NONE),
                    Some(&0),
                    Some(&1),
                    D3D12_TILE_MAPPING_FLAG_NONE,
                );
            }
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
    let snapshot = LAYOUT.get()?;
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
    let layouts: Vec<_> = buffers
        .iter()
        .map(|buffer| {
            buffer.current_layout(*snapshot).unwrap_or_else(|| {
                crate::host::fail(format_args!(
                    "XMem lost the layout for GPU buffer {:#x}",
                    buffer.base
                ))
            })
        })
        .collect();
    if overlapping_owners(layouts.iter().map(Vec::as_slice)) {
        crate::host::fail(format_args!(
            "XMem remapped two reserved GPU buffers onto the same tiles"
        ));
    }
    let prior = buffers
        .iter()
        .zip(layouts)
        .flat_map(|(buffer, layout)| buffer.bind(queue, layout, new_queue))
        .collect();
    Some(InFlightTiles {
        _buffers: buffers,
        _prior: prior,
    })
}
