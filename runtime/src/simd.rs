//! SIMD-optimized primitives
//!
//! Provides fast implementations of common operations using CPU SIMD instructions.
//! Falls back to scalar implementations on unsupported platforms.

// =============================================================================
// memchr - find byte in slice
// =============================================================================

/// Find first occurrence of byte in slice (SIMD-accelerated)
#[inline]
pub fn memchr(needle: u8, haystack: &[u8]) -> Option<usize> {
    // Use SIMD for larger buffers
    #[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
    {
        if haystack.len() >= 16 {
            return unsafe { memchr_sse2(needle, haystack) };
        }
    }

    #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
    {
        if haystack.len() >= 32 {
            return unsafe { memchr_avx2(needle, haystack) };
        }
    }

    // Scalar fallback
    memchr_scalar(needle, haystack)
}

/// Scalar implementation
#[inline]
fn memchr_scalar(needle: u8, haystack: &[u8]) -> Option<usize> {
    haystack.iter().position(|&b| b == needle)
}

/// SSE2 implementation (128-bit, 16 bytes at a time)
#[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
#[inline]
unsafe fn memchr_sse2(needle: u8, haystack: &[u8]) -> Option<usize> {
    use std::arch::x86_64::*;

    let len = haystack.len();
    let ptr = haystack.as_ptr();

    // Broadcast needle to all lanes
    let needle_vec = _mm_set1_epi8(needle as i8);

    let mut i = 0;

    // Process 16 bytes at a time
    while i + 16 <= len {
        let chunk = _mm_loadu_si128(ptr.add(i) as *const __m128i);
        let cmp = _mm_cmpeq_epi8(chunk, needle_vec);
        let mask = _mm_movemask_epi8(cmp) as u32;

        if mask != 0 {
            return Some(i + mask.trailing_zeros() as usize);
        }
        i += 16;
    }

    // Handle remaining bytes
    while i < len {
        if *ptr.add(i) == needle {
            return Some(i);
        }
        i += 1;
    }

    None
}

/// AVX2 implementation (256-bit, 32 bytes at a time)
#[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
#[inline]
unsafe fn memchr_avx2(needle: u8, haystack: &[u8]) -> Option<usize> {
    use std::arch::x86_64::*;

    let len = haystack.len();
    let ptr = haystack.as_ptr();

    // Broadcast needle to all 32 lanes
    let needle_vec = _mm256_set1_epi8(needle as i8);

    let mut i = 0;

    // Process 32 bytes at a time
    while i + 32 <= len {
        let chunk = _mm256_loadu_si256(ptr.add(i) as *const __m256i);
        let cmp = _mm256_cmpeq_epi8(chunk, needle_vec);
        let mask = _mm256_movemask_epi8(cmp) as u32;

        if mask != 0 {
            return Some(i + mask.trailing_zeros() as usize);
        }
        i += 32;
    }

    // Process remaining 16 bytes with SSE2 if available
    #[cfg(target_feature = "sse2")]
    {
        let needle_vec_sse = _mm_set1_epi8(needle as i8);
        while i + 16 <= len {
            let chunk = _mm_loadu_si128(ptr.add(i) as *const __m128i);
            let cmp = _mm_cmpeq_epi8(chunk, needle_vec_sse);
            let mask = _mm_movemask_epi8(cmp) as u32;

            if mask != 0 {
                return Some(i + mask.trailing_zeros() as usize);
            }
            i += 16;
        }
    }

    // Handle remaining bytes
    while i < len {
        if *ptr.add(i) == needle {
            return Some(i);
        }
        i += 1;
    }

    None
}

// =============================================================================
// memchr2 - find first of two bytes
// =============================================================================

/// Find first occurrence of either byte in slice
#[inline]
pub fn memchr2(needle1: u8, needle2: u8, haystack: &[u8]) -> Option<usize> {
    #[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
    {
        if haystack.len() >= 16 {
            return unsafe { memchr2_sse2(needle1, needle2, haystack) };
        }
    }

    // Scalar fallback
    haystack.iter().position(|&b| b == needle1 || b == needle2)
}

#[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
#[inline]
unsafe fn memchr2_sse2(needle1: u8, needle2: u8, haystack: &[u8]) -> Option<usize> {
    use std::arch::x86_64::*;

    let len = haystack.len();
    let ptr = haystack.as_ptr();

    let needle1_vec = _mm_set1_epi8(needle1 as i8);
    let needle2_vec = _mm_set1_epi8(needle2 as i8);

    let mut i = 0;

    while i + 16 <= len {
        let chunk = _mm_loadu_si128(ptr.add(i) as *const __m128i);
        let cmp1 = _mm_cmpeq_epi8(chunk, needle1_vec);
        let cmp2 = _mm_cmpeq_epi8(chunk, needle2_vec);
        let cmp = _mm_or_si128(cmp1, cmp2);
        let mask = _mm_movemask_epi8(cmp) as u32;

        if mask != 0 {
            return Some(i + mask.trailing_zeros() as usize);
        }
        i += 16;
    }

    while i < len {
        let b = *ptr.add(i);
        if b == needle1 || b == needle2 {
            return Some(i);
        }
        i += 1;
    }

    None
}

// =============================================================================
// memmem - find substring (for \r\n)
// =============================================================================

/// Find \r\n in buffer (optimized for HTTP parsing)
#[inline]
pub fn find_crlf(haystack: &[u8]) -> Option<usize> {
    #[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
    {
        if haystack.len() >= 16 {
            return unsafe { find_crlf_sse2(haystack) };
        }
    }

    // Scalar fallback
    find_crlf_scalar(haystack)
}

#[inline]
fn find_crlf_scalar(haystack: &[u8]) -> Option<usize> {
    let mut i = 0;
    while i + 1 < haystack.len() {
        if haystack[i] == b'\r' && haystack[i + 1] == b'\n' {
            return Some(i);
        }
        i += 1;
    }
    None
}

#[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
#[inline]
unsafe fn find_crlf_sse2(haystack: &[u8]) -> Option<usize> {
    use std::arch::x86_64::*;

    let len = haystack.len();
    let ptr = haystack.as_ptr();

    let cr_vec = _mm_set1_epi8(b'\r' as i8);

    let mut i = 0;

    // First find \r, then check if next byte is \n
    while i + 16 <= len {
        let chunk = _mm_loadu_si128(ptr.add(i) as *const __m128i);
        let cmp = _mm_cmpeq_epi8(chunk, cr_vec);
        let mut mask = _mm_movemask_epi8(cmp) as u32;

        while mask != 0 {
            let bit_pos = mask.trailing_zeros() as usize;
            let pos = i + bit_pos;

            if pos + 1 < len && *ptr.add(pos + 1) == b'\n' {
                return Some(pos);
            }

            // Clear this bit and continue
            mask &= mask - 1;
        }
        i += 16;
    }

    // Handle remaining bytes
    find_crlf_scalar(&haystack[i..]).map(|p| p + i)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memchr_found() {
        let data = b"hello world";
        assert_eq!(memchr(b'w', data), Some(6));
        assert_eq!(memchr(b'h', data), Some(0));
        assert_eq!(memchr(b'd', data), Some(10));
    }

    #[test]
    fn test_memchr_not_found() {
        let data = b"hello world";
        assert_eq!(memchr(b'z', data), None);
    }

    #[test]
    fn test_memchr_empty() {
        assert_eq!(memchr(b'a', b""), None);
    }

    #[test]
    fn test_memchr_large() {
        // Test SIMD path
        let mut data = vec![b'x'; 1000];
        data[500] = b'y';
        assert_eq!(memchr(b'y', &data), Some(500));

        data[999] = b'z';
        assert_eq!(memchr(b'z', &data), Some(999));
    }

    #[test]
    fn test_memchr2() {
        let data = b"hello world";
        assert_eq!(memchr2(b'o', b'w', data), Some(4)); // 'o' comes first
        assert_eq!(memchr2(b'z', b'w', data), Some(6)); // only 'w' found
        assert_eq!(memchr2(b'z', b'q', data), None);
    }

    #[test]
    fn test_find_crlf() {
        assert_eq!(find_crlf(b"hello\r\nworld"), Some(5));
        assert_eq!(find_crlf(b"\r\n"), Some(0));
        assert_eq!(find_crlf(b"no crlf"), None);
        assert_eq!(find_crlf(b"just\r"), None);
        assert_eq!(find_crlf(b"just\n"), None);
    }

    #[test]
    fn test_find_crlf_large() {
        let mut data = vec![b'x'; 1000];
        data[500] = b'\r';
        data[501] = b'\n';
        assert_eq!(find_crlf(&data), Some(500));
    }

    #[test]
    fn test_find_crlf_at_end() {
        let mut data = vec![b'x'; 100];
        data[98] = b'\r';
        data[99] = b'\n';
        assert_eq!(find_crlf(&data), Some(98));
    }
}
