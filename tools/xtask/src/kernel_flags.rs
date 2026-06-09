use std::env;
use std::path::Path;
use std::process::Command;

const RUSTFLAGS_SEPARATOR: char = '\x1f';
const KERNEL_RUSTFLAGS: &[&str] = &[
    "-C",
    "code-model=kernel",
    "-C",
    "relocation-model=static",
    "-C",
    "link-arg=-Tlinker/kernel-x86_64.ld",
    "-C",
    "panic=abort",
    "-C",
    "target-feature=-sse,-sse2,-sse3,-ssse3,-sse4.1,-sse4.2,-avx,-avx2",
];

pub fn apply_kernel_rustflags(command: &mut Command, root: &Path) {
    command.env(
        "CARGO_ENCODED_RUSTFLAGS",
        encoded_kernel_rustflags(root, existing_encoded_rustflags().as_deref()),
    );
}

fn existing_encoded_rustflags() -> Option<String> {
    match env::var("CARGO_ENCODED_RUSTFLAGS") {
        Ok(value) if !value.is_empty() => Some(value),
        _empty_or_missing => None,
    }
}

fn encoded_kernel_rustflags(root: &Path, existing: Option<&str>) -> String {
    let root = root.display();
    let kernel_flags = KERNEL_RUSTFLAGS.join("\x1f");
    let remap = format!("--remap-path-prefix{}{root}=.", RUSTFLAGS_SEPARATOR);
    match existing {
        Some(existing) => {
            format!("{existing}{RUSTFLAGS_SEPARATOR}{kernel_flags}{RUSTFLAGS_SEPARATOR}{remap}")
        }
        None => format!("{kernel_flags}{RUSTFLAGS_SEPARATOR}{remap}"),
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{RUSTFLAGS_SEPARATOR, encoded_kernel_rustflags};

    #[test]
    fn kernel_rustflags_include_workspace_path_remap() {
        assert_eq!(
            encoded_kernel_rustflags(Path::new("/work/aesynx"), None),
            format!(
                "-C{RUSTFLAGS_SEPARATOR}code-model=kernel{RUSTFLAGS_SEPARATOR}-C{RUSTFLAGS_SEPARATOR}relocation-model=static{RUSTFLAGS_SEPARATOR}-C{RUSTFLAGS_SEPARATOR}link-arg=-Tlinker/kernel-x86_64.ld{RUSTFLAGS_SEPARATOR}-C{RUSTFLAGS_SEPARATOR}panic=abort{RUSTFLAGS_SEPARATOR}-C{RUSTFLAGS_SEPARATOR}target-feature=-sse,-sse2,-sse3,-ssse3,-sse4.1,-sse4.2,-avx,-avx2{RUSTFLAGS_SEPARATOR}--remap-path-prefix{RUSTFLAGS_SEPARATOR}/work/aesynx=."
            )
        );
    }

    #[test]
    fn kernel_rustflags_preserve_existing_encoded_flags() {
        assert_eq!(
            encoded_kernel_rustflags(
                Path::new("/work/aesynx"),
                Some("-C\x1fforce-frame-pointers=yes")
            ),
            format!(
                "-C{RUSTFLAGS_SEPARATOR}force-frame-pointers=yes{RUSTFLAGS_SEPARATOR}-C{RUSTFLAGS_SEPARATOR}code-model=kernel{RUSTFLAGS_SEPARATOR}-C{RUSTFLAGS_SEPARATOR}relocation-model=static{RUSTFLAGS_SEPARATOR}-C{RUSTFLAGS_SEPARATOR}link-arg=-Tlinker/kernel-x86_64.ld{RUSTFLAGS_SEPARATOR}-C{RUSTFLAGS_SEPARATOR}panic=abort{RUSTFLAGS_SEPARATOR}-C{RUSTFLAGS_SEPARATOR}target-feature=-sse,-sse2,-sse3,-ssse3,-sse4.1,-sse4.2,-avx,-avx2{RUSTFLAGS_SEPARATOR}--remap-path-prefix{RUSTFLAGS_SEPARATOR}/work/aesynx=."
            )
        );
    }

    #[test]
    fn kernel_rustflags_disable_simd_until_fpu_context_exists() {
        let flags = encoded_kernel_rustflags(Path::new("/work/aesynx"), None);

        assert!(
            flags.contains("target-feature=-sse,-sse2,-sse3,-ssse3,-sse4.1,-sse4.2,-avx,-avx2")
        );
    }
}
