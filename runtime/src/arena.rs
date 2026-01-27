//! Arena Allocator for PHPRS Runtime
//!
//! High-performance bump allocator for per-request allocations.
//! All allocations are freed at once when the arena is reset.
//!
//! Key features:
//! - O(1) allocation (just bump a pointer)
//! - O(1) deallocation (reset the bump pointer)
//! - Cache-friendly sequential memory layout
//! - Thread-local arenas for lock-free allocation

use std::alloc::{alloc, dealloc, Layout};
use std::cell::RefCell;
use std::ptr::NonNull;

// =============================================================================
// Constants
// =============================================================================

/// Default chunk size: 64KB (fits in L2 cache)
const DEFAULT_CHUNK_SIZE: usize = 64 * 1024;

/// Maximum alignment we support
const MAX_ALIGN: usize = 16;

// =============================================================================
// Chunk - individual memory block
// =============================================================================

struct Chunk {
    /// Start of allocated memory
    data: NonNull<u8>,
    /// Total capacity
    capacity: usize,
    /// Current position (bytes used)
    pos: usize,
}

impl Chunk {
    fn new(capacity: usize) -> Self {
        let layout = Layout::from_size_align(capacity, MAX_ALIGN)
            .expect("Invalid layout");

        let data = unsafe {
            let ptr = alloc(layout);
            if ptr.is_null() {
                std::alloc::handle_alloc_error(layout);
            }
            NonNull::new_unchecked(ptr)
        };

        Chunk {
            data,
            capacity,
            pos: 0,
        }
    }

    /// Allocate bytes from this chunk, returns None if not enough space
    #[inline]
    fn alloc(&mut self, size: usize, align: usize) -> Option<NonNull<u8>> {
        let aligned_pos = (self.pos + align - 1) & !(align - 1);
        let new_pos = aligned_pos + size;

        if new_pos > self.capacity {
            return None;
        }

        self.pos = new_pos;

        unsafe {
            Some(NonNull::new_unchecked(self.data.as_ptr().add(aligned_pos)))
        }
    }

    /// Reset chunk for reuse
    #[inline]
    fn reset(&mut self) {
        self.pos = 0;
    }

    /// Remaining space in chunk
    #[inline]
    fn remaining(&self) -> usize {
        self.capacity - self.pos
    }
}

impl Drop for Chunk {
    fn drop(&mut self) {
        let layout = Layout::from_size_align(self.capacity, MAX_ALIGN)
            .expect("Invalid layout");
        unsafe {
            dealloc(self.data.as_ptr(), layout);
        }
    }
}

// =============================================================================
// Arena - the main allocator
// =============================================================================

/// Bump allocator for fast, temporary allocations
///
/// # Example
/// ```ignore
/// let arena = Arena::new();
/// let ptr = arena.alloc::<i64>();
/// // Use ptr...
/// arena.reset(); // All allocations freed instantly
/// ```
pub struct Arena {
    /// Active chunks
    chunks: Vec<Chunk>,
    /// Index of current chunk
    current: usize,
    /// Default size for new chunks
    chunk_size: usize,
    /// Statistics: total bytes allocated
    total_allocated: usize,
    /// Statistics: number of allocations
    alloc_count: usize,
}

impl Arena {
    /// Create new arena with default chunk size (64KB)
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_CHUNK_SIZE)
    }

    /// Create arena with custom chunk size
    pub fn with_capacity(chunk_size: usize) -> Self {
        let chunk_size = chunk_size.max(1024); // Minimum 1KB
        Arena {
            chunks: vec![Chunk::new(chunk_size)],
            current: 0,
            chunk_size,
            total_allocated: 0,
            alloc_count: 0,
        }
    }

    /// Allocate memory for a type T
    #[inline]
    pub fn alloc<T>(&mut self) -> NonNull<T> {
        self.alloc_layout(Layout::new::<T>()).cast()
    }

    /// Allocate and initialize a value
    #[inline]
    pub fn alloc_val<T>(&mut self, val: T) -> &mut T {
        let ptr = self.alloc::<T>();
        unsafe {
            ptr.as_ptr().write(val);
            &mut *ptr.as_ptr()
        }
    }

    /// Allocate a slice of T with given length
    #[inline]
    pub fn alloc_slice<T>(&mut self, len: usize) -> NonNull<[T]> {
        let layout = Layout::array::<T>(len).expect("Invalid array layout");
        let ptr = self.alloc_layout(layout);
        unsafe {
            NonNull::new_unchecked(std::ptr::slice_from_raw_parts_mut(
                ptr.as_ptr() as *mut T,
                len,
            ))
        }
    }

    /// Allocate bytes with given layout
    #[inline]
    pub fn alloc_layout(&mut self, layout: Layout) -> NonNull<u8> {
        let size = layout.size();
        let align = layout.align().min(MAX_ALIGN);

        self.total_allocated += size;
        self.alloc_count += 1;

        // Try current chunk first
        if let Some(ptr) = self.chunks[self.current].alloc(size, align) {
            return ptr;
        }

        // Need a new chunk
        self.grow(size, align)
    }

    /// Allocate raw bytes
    #[inline]
    pub fn alloc_bytes(&mut self, size: usize) -> NonNull<u8> {
        self.alloc_layout(Layout::from_size_align(size, 1).unwrap())
    }

    /// Grow arena with new chunk
    #[cold]
    fn grow(&mut self, size: usize, align: usize) -> NonNull<u8> {
        // Check if we have an unused chunk that fits
        for i in (self.current + 1)..self.chunks.len() {
            if self.chunks[i].remaining() >= size + align {
                self.current = i;
                return self.chunks[i].alloc(size, align).unwrap();
            }
        }

        // Allocate new chunk (at least chunk_size or size + padding)
        let new_size = self.chunk_size.max(size + MAX_ALIGN);
        let mut chunk = Chunk::new(new_size);
        let ptr = chunk.alloc(size, align).unwrap();

        self.chunks.push(chunk);
        self.current = self.chunks.len() - 1;

        ptr
    }

    /// Reset arena - all allocations are invalidated
    ///
    /// This is O(n) where n is number of chunks, but chunks are reused
    #[inline]
    pub fn reset(&mut self) {
        for chunk in &mut self.chunks {
            chunk.reset();
        }
        self.current = 0;
        self.total_allocated = 0;
        self.alloc_count = 0;
    }

    /// Get statistics
    pub fn stats(&self) -> ArenaStats {
        let total_capacity: usize = self.chunks.iter().map(|c| c.capacity).sum();
        ArenaStats {
            total_allocated: self.total_allocated,
            total_capacity,
            alloc_count: self.alloc_count,
            chunk_count: self.chunks.len(),
        }
    }
}

impl Default for Arena {
    fn default() -> Self {
        Self::new()
    }
}

// Arena is not Send/Sync by default (contains raw pointers)
// For thread-local usage, this is fine

// =============================================================================
// Statistics
// =============================================================================

#[derive(Debug, Clone, Copy)]
pub struct ArenaStats {
    pub total_allocated: usize,
    pub total_capacity: usize,
    pub alloc_count: usize,
    pub chunk_count: usize,
}

// =============================================================================
// Thread-local arena
// =============================================================================

thread_local! {
    static THREAD_ARENA: RefCell<Arena> = RefCell::new(Arena::new());
}

/// Allocate from thread-local arena
#[inline]
pub fn thread_alloc<T>() -> NonNull<T> {
    THREAD_ARENA.with(|arena| arena.borrow_mut().alloc::<T>())
}

/// Allocate and initialize value in thread-local arena
#[inline]
pub fn thread_alloc_val<T>(val: T) -> &'static mut T {
    THREAD_ARENA.with(|arena| {
        let ptr = arena.borrow_mut().alloc::<T>();
        unsafe {
            ptr.as_ptr().write(val);
            &mut *ptr.as_ptr()
        }
    })
}

/// Reset thread-local arena (call at end of request)
#[inline]
pub fn thread_arena_reset() {
    THREAD_ARENA.with(|arena| arena.borrow_mut().reset());
}

/// Get thread-local arena statistics
pub fn thread_arena_stats() -> ArenaStats {
    THREAD_ARENA.with(|arena| arena.borrow().stats())
}

// =============================================================================
// C ABI exports
// =============================================================================

/// Create new arena
#[no_mangle]
pub extern "C" fn rt_arena_new() -> *mut Arena {
    Box::into_raw(Box::new(Arena::new()))
}

/// Create arena with custom chunk size
#[no_mangle]
pub extern "C" fn rt_arena_with_capacity(chunk_size: usize) -> *mut Arena {
    Box::into_raw(Box::new(Arena::with_capacity(chunk_size)))
}

/// Allocate bytes from arena
#[no_mangle]
pub extern "C" fn rt_arena_alloc(arena: &mut Arena, size: usize, align: usize) -> *mut u8 {
    let layout = Layout::from_size_align(size, align.clamp(1, MAX_ALIGN))
        .expect("Invalid layout");
    arena.alloc_layout(layout).as_ptr()
}

/// Reset arena
#[no_mangle]
pub extern "C" fn rt_arena_reset(arena: &mut Arena) {
    arena.reset();
}

/// Free arena
///
/// # Safety
/// The pointer must have been allocated by `rt_arena_new` and must not be used after this call.
#[no_mangle]
pub unsafe extern "C" fn rt_arena_free(arena: *mut Arena) {
    if !arena.is_null() {
        drop(Box::from_raw(arena));
    }
}

/// Allocate from thread-local arena
#[no_mangle]
pub extern "C" fn rt_thread_alloc(size: usize) -> *mut u8 {
    THREAD_ARENA.with(|arena| {
        arena.borrow_mut().alloc_bytes(size).as_ptr()
    })
}

/// Reset thread-local arena
#[no_mangle]
pub extern "C" fn rt_thread_arena_reset() {
    thread_arena_reset();
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_allocation() {
        let mut arena = Arena::new();

        let p1 = arena.alloc::<i64>();
        let p2 = arena.alloc::<i64>();

        // Pointers should be different
        assert_ne!(p1.as_ptr(), p2.as_ptr());

        // Should be sequential (with alignment)
        let diff = (p2.as_ptr() as usize) - (p1.as_ptr() as usize);
        assert_eq!(diff, 8); // i64 is 8 bytes
    }

    #[test]
    fn test_alloc_val() {
        let mut arena = Arena::new();

        let x_ptr = arena.alloc::<i64>();
        unsafe { *x_ptr.as_ptr() = 42; }

        let y_ptr = arena.alloc::<i64>();
        unsafe { *y_ptr.as_ptr() = 100; }

        assert_eq!(unsafe { *x_ptr.as_ptr() }, 42);
        assert_eq!(unsafe { *y_ptr.as_ptr() }, 100);
    }

    #[test]
    fn test_alloc_slice() {
        let mut arena = Arena::new();

        let slice_ptr = arena.alloc_slice::<i32>(10);
        let slice = unsafe { slice_ptr.as_ref() };

        assert_eq!(slice.len(), 10);
    }

    #[test]
    fn test_reset() {
        let mut arena = Arena::new();

        // Allocate some memory
        for _ in 0..100 {
            arena.alloc::<[u8; 1024]>();
        }

        let stats_before = arena.stats();
        assert!(stats_before.total_allocated > 0);

        arena.reset();

        let stats_after = arena.stats();
        assert_eq!(stats_after.total_allocated, 0);
        assert_eq!(stats_after.alloc_count, 0);
        // Chunks are preserved
        assert_eq!(stats_after.chunk_count, stats_before.chunk_count);
    }

    #[test]
    fn test_grow() {
        let mut arena = Arena::with_capacity(1024);

        // Force multiple chunks
        for _ in 0..100 {
            arena.alloc::<[u8; 256]>();
        }

        let stats = arena.stats();
        assert!(stats.chunk_count > 1);
    }

    #[test]
    fn test_alignment() {
        let mut arena = Arena::new();

        // Allocate byte
        let _ = arena.alloc::<u8>();

        // Next i64 should be aligned
        let p = arena.alloc::<i64>();
        assert_eq!(p.as_ptr() as usize % 8, 0);
    }

    #[test]
    fn test_large_allocation() {
        let mut arena = Arena::with_capacity(1024);

        // Allocate more than chunk size
        let large = arena.alloc_slice::<u8>(10000);
        let slice = unsafe { large.as_ref() };

        assert_eq!(slice.len(), 10000);
    }

    #[test]
    fn test_thread_local() {
        let p1 = thread_alloc::<i64>();
        let p2 = thread_alloc::<i64>();

        assert_ne!(p1.as_ptr(), p2.as_ptr());

        thread_arena_reset();

        let stats = thread_arena_stats();
        assert_eq!(stats.total_allocated, 0);
    }

    #[test]
    fn test_many_small_allocations() {
        let mut arena = Arena::new();

        // Simulate typical request with many small allocations
        for _ in 0..10000 {
            arena.alloc::<i64>();
        }

        let stats = arena.stats();
        assert_eq!(stats.alloc_count, 10000);
        assert_eq!(stats.total_allocated, 10000 * 8);
    }
}
