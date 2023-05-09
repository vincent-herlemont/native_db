use shortcut_assert_fs::TmpFs;
use std::sync::Once;

#[allow(dead_code)]
static INSTALL_ONCE: Once = Once::new();

#[allow(dead_code)]
pub fn init() -> TmpFs {
    let mut _installed = false;

    INSTALL_ONCE.call_once(|| {
        #[cfg(feature = "eyre_support")]
        {
            color_eyre::install().unwrap();
            _installed = true;
        }
    });

    TmpFs::new().unwrap()
}
