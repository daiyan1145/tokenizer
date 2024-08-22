pub(crate) const NEW_LINE: &'static str =
if cfg!(target_os = "windows") {
    "\r\n"
} else if cfg!(target_os = "linux") {
    "\n"
} else {
    unreachable!()
};