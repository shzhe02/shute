use std::arch::x86_64::{
    __m256, _mm256_add_ps, _mm256_min_ps, _mm256_permute_ps, _mm256_permute2f128_ps, _mm256_set_ps,
    _mm256_set1_ps, _mm256_setzero_ps, _mm256_store_ps,
};

use rayon::iter::{IntoParallelIterator, ParallelBridge, ParallelIterator};

#[inline]
fn swap4(input: __m256) -> __m256 {
    unsafe { _mm256_permute2f128_ps::<0b00000001>(input, input) }
}
#[inline]
fn swap2(input: __m256) -> __m256 {
    unsafe { _mm256_permute_ps::<0b01001110>(input) }
}
#[inline]
fn swap1(input: __m256) -> __m256 {
    unsafe { _mm256_permute_ps::<0b10110001>(input) }
}
#[inline]
fn min8(a: __m256, b: __m256) -> __m256 {
    unsafe { _mm256_min_ps(a, b) }
}
#[inline]
fn add8(a: __m256, b: __m256) -> __m256 {
    unsafe { _mm256_add_ps(a, b) }
}

struct PtrWrapper(*mut f32);

unsafe impl Sync for PtrWrapper {}

pub unsafe fn cpu_compute(data: &[f32], dim: u32) -> Vec<f32> {
    use std::time::Instant;

    let now = Instant::now();
    let mut output = vec![0.0; (dim * dim) as usize];
    let output_ptr = PtrWrapper(output.as_mut_ptr());
    // vectors per input column
    let na = dim.div_ceil(8);

    // input data, padded, converted to vectors
    let mut vd = vec![unsafe { _mm256_setzero_ps() }; (na * dim) as usize];
    // input data, transposed, padded, converted to vectors
    let mut vt = vec![unsafe { _mm256_setzero_ps() }; (na * dim) as usize];

    vd.chunks_mut(dim as usize)
        .zip(vt.chunks_mut(dim as usize))
        .enumerate()
        .par_bridge()
        .for_each(|(ja, (chunk_vd, chunk_vt))| {
            for i in 0..dim {
                let mut vvd = vec![f32::MAX; 8];
                let mut vvt = vec![f32::MAX; 8];
                for jb in 0..8usize {
                    let j = ja * 8 + jb;
                    if (j as u32) < dim {
                        vvd[jb] = data[dim as usize * j + i as usize];
                        vvt[jb] = data[(dim * i) as usize + j];
                    }
                }
                chunk_vd[i as usize] = unsafe {
                    _mm256_set_ps(
                        vvd[7], vvd[6], vvd[5], vvd[4], vvd[3], vvd[2], vvd[1], vvd[0],
                    )
                };
                chunk_vt[i as usize] = unsafe {
                    _mm256_set_ps(
                        vvt[7], vvt[6], vvt[5], vvt[4], vvt[3], vvt[2], vvt[1], vvt[0],
                    )
                };
            }
        });

    (0..na).into_par_iter().for_each(|ia| {
        let _ = &output_ptr;
        for ja in 0..na {
            let now = Instant::now();
            let mut vv000 = unsafe { _mm256_set1_ps(f32::MAX) };
            let mut vv001 = unsafe { _mm256_set1_ps(f32::MAX) };
            let mut vv010 = unsafe { _mm256_set1_ps(f32::MAX) };
            let mut vv011 = unsafe { _mm256_set1_ps(f32::MAX) };
            let mut vv100 = unsafe { _mm256_set1_ps(f32::MAX) };
            let mut vv101 = unsafe { _mm256_set1_ps(f32::MAX) };
            let mut vv110 = unsafe { _mm256_set1_ps(f32::MAX) };
            let mut vv111 = unsafe { _mm256_set1_ps(f32::MAX) };
            for k in 0..dim {
                let a000 = vd[(dim * ia + k) as usize];
                let b000 = vt[(dim * ja + k) as usize];
                let a100 = swap4(a000);
                let a010 = swap2(a000);
                let a110 = swap2(a100);
                let b001 = swap1(b000);
                vv000 = min8(vv000, add8(a000, b000));
                vv001 = min8(vv001, add8(a000, b001));
                vv010 = min8(vv010, add8(a010, b000));
                vv011 = min8(vv011, add8(a010, b001));
                vv100 = min8(vv100, add8(a100, b000));
                vv101 = min8(vv101, add8(a100, b001));
                vv110 = min8(vv110, add8(a110, b000));
                vv111 = min8(vv111, add8(a110, b001));
            }
            let mut vv = vec![vv000, vv001, vv010, vv011, vv100, vv101, vv110, vv111];
            for kb in (1..8usize).step_by(2) {
                vv[kb] = swap1(vv[kb]);
            }
            for jb in 0..8 {
                for ib in 0..8 {
                    let i = ib + ia * 8;
                    let j = jb + ja * 8;
                    if j < dim && i < dim {
                        let mut temp = [0.0; 8];
                        unsafe {
                            _mm256_store_ps(temp.as_mut_ptr(), vv[(ib ^ jb) as usize]);
                            *output_ptr.0.add((dim * i + j) as usize) =
                                *temp.get_unchecked(jb as usize);
                        }
                    }
                }
            }
            let elapsed = now.elapsed();
            println!("completed in {:?}", elapsed);
        }
    });
    output
}
