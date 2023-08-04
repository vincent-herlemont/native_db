use shortcut_assert_fs::TmpFs;

#[allow(dead_code)]
pub fn init() -> TmpFs {

    TmpFs::new().unwrap()
}
