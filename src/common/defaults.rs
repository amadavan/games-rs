use std::sync::LazyLock;

use indicatif::ProgressStyle;

pub static PB_STYLE: LazyLock<ProgressStyle> = LazyLock::new(|| {
    ProgressStyle::with_template(
        "{prefix} [{elapsed_precise}] [{wide_bar:1}] {pos:>7}/{len:7} ({eta_precise})",
    )
    .unwrap()
});
