use std::{
    ffi::{c_int, c_void},
    ptr::NonNull,
};

use libsqlite3_sys::{
    sqlite3, sqlite3_context, sqlite3_create_function, sqlite3_result_int64, sqlite3_result_text,
    sqlite3_value, sqlite3_value_int64, sqlite3_value_type, SQLITE_INTEGER,
};
use std::io::Write;

pub fn load_funcs(sql: NonNull<sqlite3>) {
    unsafe {
        let sqlite_check = sqlite3_create_function(
            sql.as_ptr(),
            "minutes\0".as_ptr().cast(),
            1,
            libsqlite3_sys::SQLITE_UTF8,
            std::ptr::null_mut(),
            Some(minutes),
            None,
            None,
        );

        assert_eq!(sqlite_check, 0, "function create failed");

        let sqlite_check = sqlite3_create_function(
            sql.as_ptr(),
            "hours\0".as_ptr().cast(),
            1,
            libsqlite3_sys::SQLITE_UTF8,
            std::ptr::null_mut(),
            Some(hours),
            None,
            None,
        );

        assert_eq!(sqlite_check, 0, "function create failed");

        let sqlite_check = sqlite3_create_function(
            sql.as_ptr(),
            "days\0".as_ptr().cast(),
            1,
            libsqlite3_sys::SQLITE_UTF8,
            std::ptr::null_mut(),
            Some(days),
            None,
            None,
        );

        assert_eq!(sqlite_check, 0, "function create failed");

        let sqlite_check = sqlite3_create_function(
            sql.as_ptr(),
            "duration\0".as_ptr().cast(),
            1,
            libsqlite3_sys::SQLITE_UTF8,
            std::ptr::null_mut(),
            Some(duration_repr),
            None,
            None,
        );

        assert_eq!(sqlite_check, 0, "function create failed");
    }
}

unsafe fn is_duration(value: *mut sqlite3_value) -> bool {
    sqlite3_value_type(value) == SQLITE_INTEGER
}

unsafe extern "C" fn minutes(
    ctx: *mut sqlite3_context,
    len: c_int,
    values: *mut *mut sqlite3_value,
) {
    if len != 1 || !is_duration(values.read()) {
        return;
    }

    let seconds = sqlite3_value_int64(values.read());
    let minutes = (seconds / 60) % 60;

    sqlite3_result_int64(ctx, minutes);
}

unsafe extern "C" fn hours(ctx: *mut sqlite3_context, len: c_int, values: *mut *mut sqlite3_value) {
    if len != 1 || !is_duration(values.read()) {
        return;
    }

    let seconds = sqlite3_value_int64(values.read());
    let hours = (seconds / 3600) % 24;

    sqlite3_result_int64(ctx, hours);
}

unsafe extern "C" fn days(ctx: *mut sqlite3_context, len: c_int, values: *mut *mut sqlite3_value) {
    if len != 1 || !is_duration(values.read()) {
        return;
    }

    let seconds = sqlite3_value_int64(values.read());
    let days = seconds / 86400;

    sqlite3_result_int64(ctx, days);
}

unsafe extern "C" fn duration_repr(
    ctx: *mut sqlite3_context,
    len: c_int,
    values: *mut *mut sqlite3_value,
) {
    if len != 1 || !is_duration(values.read()) {
        return;
    }

    let mut dur = sqlite3_value_int64(values.read());

    let seconds = dur % 60;
    dur /= 60;

    let minutes = dur % 60;
    dur /= 60;

    let hours = dur % 24;
    dur /= 24;

    let mut buf = Vec::<u8>::with_capacity(32);
    let mut force = false;

    if dur != 0 {
        let _ = write!(buf, "{}d ", dur);
        force = true;
    }

    if hours != 0 || force {
        let _ = write!(buf, "{}h ", hours);
        force = true;
    }

    if minutes != 0 || force {
        let _ = write!(buf, "{}m ", minutes);
    }

    let _ = write!(buf, "{}s", seconds);

    buf.shrink_to_fit();

    let slice = buf.leak();

    sqlite3_result_text(
        ctx,
        slice.as_ptr().cast(),
        slice.len() as c_int,
        Some(slice_free),
    );
}

unsafe extern "C" fn slice_free(ptr: *mut c_void) {
    let buf = std::ffi::CStr::from_ptr(ptr.cast());
    let len = buf.count_bytes();

    Vec::from_raw_parts(ptr, len, len);
}
