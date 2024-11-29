pub fn option_to_ptr<T>(opt: Option<&T>) -> *const T {
    opt.map_or(std::ptr::null(), |v| v as *const _)
}
