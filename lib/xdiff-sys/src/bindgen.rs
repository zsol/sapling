/*
 * Copyright (c) Facebook, Inc. and its affiliates.
 *
 * This software may be used and distributed according to the terms of the
 * GNU General Public License version 2.
 */

/* automatically generated by rust-bindgen */

pub const _STDINT_H: u32 = 1;
pub const _FEATURES_H: u32 = 1;
pub const __USE_ANSI: u32 = 1;
pub const _BSD_SOURCE: u32 = 1;
pub const _SVID_SOURCE: u32 = 1;
pub const __USE_ISOC99: u32 = 1;
pub const __USE_ISOC95: u32 = 1;
pub const _POSIX_SOURCE: u32 = 1;
pub const _POSIX_C_SOURCE: u32 = 200809;
pub const __USE_POSIX_IMPLICITLY: u32 = 1;
pub const __USE_POSIX: u32 = 1;
pub const __USE_POSIX2: u32 = 1;
pub const __USE_POSIX199309: u32 = 1;
pub const __USE_POSIX199506: u32 = 1;
pub const __USE_XOPEN2K: u32 = 1;
pub const __USE_XOPEN2K8: u32 = 1;
pub const _ATFILE_SOURCE: u32 = 1;
pub const __USE_MISC: u32 = 1;
pub const __USE_BSD: u32 = 1;
pub const __USE_SVID: u32 = 1;
pub const __USE_ATFILE: u32 = 1;
pub const __USE_FORTIFY_LEVEL: u32 = 0;
pub const _STDC_PREDEF_H: u32 = 1;
pub const __STDC_IEC_559__: u32 = 1;
pub const __STDC_IEC_559_COMPLEX__: u32 = 1;
pub const __STDC_ISO_10646__: u32 = 201103;
pub const __STDC_NO_THREADS__: u32 = 1;
pub const __GNU_LIBRARY__: u32 = 6;
pub const __GLIBC__: u32 = 2;
pub const __GLIBC_MINOR__: u32 = 17;
pub const __GLIBC_HAVE_LONG_LONG: u32 = 1;
pub const _SYS_CDEFS_H: u32 = 1;
pub const __WORDSIZE: u32 = 64;
pub const __WORDSIZE_TIME64_COMPAT32: u32 = 1;
pub const __SYSCALL_WORDSIZE: u32 = 64;
pub const _BITS_WCHAR_H: u32 = 1;
pub const __WCHAR_MIN: i32 = -2147483648;
pub const __WCHAR_MAX: u32 = 2147483647;
pub const INT8_MIN: i32 = -128;
pub const INT16_MIN: i32 = -32768;
pub const INT32_MIN: i32 = -2147483648;
pub const INT8_MAX: u32 = 127;
pub const INT16_MAX: u32 = 32767;
pub const INT32_MAX: u32 = 2147483647;
pub const UINT8_MAX: u32 = 255;
pub const UINT16_MAX: u32 = 65535;
pub const UINT32_MAX: u32 = 4294967295;
pub const INT_LEAST8_MIN: i32 = -128;
pub const INT_LEAST16_MIN: i32 = -32768;
pub const INT_LEAST32_MIN: i32 = -2147483648;
pub const INT_LEAST8_MAX: u32 = 127;
pub const INT_LEAST16_MAX: u32 = 32767;
pub const INT_LEAST32_MAX: u32 = 2147483647;
pub const UINT_LEAST8_MAX: u32 = 255;
pub const UINT_LEAST16_MAX: u32 = 65535;
pub const UINT_LEAST32_MAX: u32 = 4294967295;
pub const INT_FAST8_MIN: i32 = -128;
pub const INT_FAST16_MIN: i64 = -9223372036854775808;
pub const INT_FAST32_MIN: i64 = -9223372036854775808;
pub const INT_FAST8_MAX: u32 = 127;
pub const INT_FAST16_MAX: u64 = 9223372036854775807;
pub const INT_FAST32_MAX: u64 = 9223372036854775807;
pub const UINT_FAST8_MAX: u32 = 255;
pub const UINT_FAST16_MAX: i32 = -1;
pub const UINT_FAST32_MAX: i32 = -1;
pub const INTPTR_MIN: i64 = -9223372036854775808;
pub const INTPTR_MAX: u64 = 9223372036854775807;
pub const UINTPTR_MAX: i32 = -1;
pub const PTRDIFF_MIN: i64 = -9223372036854775808;
pub const PTRDIFF_MAX: u64 = 9223372036854775807;
pub const SIG_ATOMIC_MIN: i32 = -2147483648;
pub const SIG_ATOMIC_MAX: u32 = 2147483647;
pub const SIZE_MAX: i32 = -1;
pub const WCHAR_MIN: i32 = -2147483648;
pub const WCHAR_MAX: u32 = 2147483647;
pub const WINT_MIN: u32 = 0;
pub const WINT_MAX: u32 = 4294967295;
pub const XDF_NEED_MINIMAL: u32 = 1;
pub const XDF_INDENT_HEURISTIC: u32 = 8388608;
pub const XDL_EMIT_BDIFFHUNK: u32 = 16;
pub type wchar_t = ::std::os::raw::c_int;
pub type int_least8_t = ::std::os::raw::c_schar;
pub type int_least16_t = ::std::os::raw::c_short;
pub type int_least32_t = ::std::os::raw::c_int;
pub type int_least64_t = ::std::os::raw::c_long;
pub type uint_least8_t = ::std::os::raw::c_uchar;
pub type uint_least16_t = ::std::os::raw::c_ushort;
pub type uint_least32_t = ::std::os::raw::c_uint;
pub type uint_least64_t = ::std::os::raw::c_ulong;
pub type int_fast8_t = ::std::os::raw::c_schar;
pub type int_fast16_t = ::std::os::raw::c_long;
pub type int_fast32_t = ::std::os::raw::c_long;
pub type int_fast64_t = ::std::os::raw::c_long;
pub type uint_fast8_t = ::std::os::raw::c_uchar;
pub type uint_fast16_t = ::std::os::raw::c_ulong;
pub type uint_fast32_t = ::std::os::raw::c_ulong;
pub type uint_fast64_t = ::std::os::raw::c_ulong;
pub type intmax_t = ::std::os::raw::c_long;
pub type uintmax_t = ::std::os::raw::c_ulong;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct s_mmfile {
    pub ptr: *mut ::std::os::raw::c_char,
    pub size: i64,
}
#[test]
fn bindgen_test_layout_s_mmfile() {
    assert_eq!(
        ::std::mem::size_of::<s_mmfile>(),
        16usize,
        concat!("Size of: ", stringify!(s_mmfile))
    );
    assert_eq!(
        ::std::mem::align_of::<s_mmfile>(),
        8usize,
        concat!("Alignment of ", stringify!(s_mmfile))
    );
}
pub type mmfile_t = s_mmfile;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct s_mmbuffer {
    pub ptr: *mut ::std::os::raw::c_char,
    pub size: i64,
}
#[test]
fn bindgen_test_layout_s_mmbuffer() {
    assert_eq!(
        ::std::mem::size_of::<s_mmbuffer>(),
        16usize,
        concat!("Size of: ", stringify!(s_mmbuffer))
    );
    assert_eq!(
        ::std::mem::align_of::<s_mmbuffer>(),
        8usize,
        concat!("Alignment of ", stringify!(s_mmbuffer))
    );
}
pub type mmbuffer_t = s_mmbuffer;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct s_xpparam {
    pub flags: u64,
}
#[test]
fn bindgen_test_layout_s_xpparam() {
    assert_eq!(
        ::std::mem::size_of::<s_xpparam>(),
        8usize,
        concat!("Size of: ", stringify!(s_xpparam))
    );
    assert_eq!(
        ::std::mem::align_of::<s_xpparam>(),
        8usize,
        concat!("Alignment of ", stringify!(s_xpparam))
    );
}
pub type xpparam_t = s_xpparam;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct s_xdemitcb {
    pub priv_: *mut ::std::os::raw::c_void,
}
#[test]
fn bindgen_test_layout_s_xdemitcb() {
    assert_eq!(
        ::std::mem::size_of::<s_xdemitcb>(),
        8usize,
        concat!("Size of: ", stringify!(s_xdemitcb))
    );
    assert_eq!(
        ::std::mem::align_of::<s_xdemitcb>(),
        8usize,
        concat!("Alignment of ", stringify!(s_xdemitcb))
    );
}
pub type xdemitcb_t = s_xdemitcb;
pub type xdl_emit_hunk_consume_func_t = ::std::option::Option<
    unsafe extern "C" fn(
        start_a: i64,
        count_a: i64,
        start_b: i64,
        count_b: i64,
        cb_data: *mut ::std::os::raw::c_void,
    ) -> ::std::os::raw::c_int,
>;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct s_xdemitconf {
    pub flags: u64,
    pub hunk_func: xdl_emit_hunk_consume_func_t,
}
#[test]
fn bindgen_test_layout_s_xdemitconf() {
    assert_eq!(
        ::std::mem::size_of::<s_xdemitconf>(),
        16usize,
        concat!("Size of: ", stringify!(s_xdemitconf))
    );
    assert_eq!(
        ::std::mem::align_of::<s_xdemitconf>(),
        8usize,
        concat!("Alignment of ", stringify!(s_xdemitconf))
    );
}
pub type xdemitconf_t = s_xdemitconf;
extern "C" {
    pub fn xdl_mmfile_first(mmf: *mut mmfile_t, size: *mut i64) -> *mut ::std::os::raw::c_void;
}
extern "C" {
    pub fn xdl_mmfile_size(mmf: *mut mmfile_t) -> i64;
}
extern "C" {
    pub fn xdl_diff(
        mf1: *mut mmfile_t,
        mf2: *mut mmfile_t,
        xpp: *const xpparam_t,
        xecfg: *const xdemitconf_t,
        ecb: *mut xdemitcb_t,
    ) -> ::std::os::raw::c_int;
}
