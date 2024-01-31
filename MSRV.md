# Minimum Supported Rust Version

| `pixels` version | `rustc` version |
|------------------|-----------------|
| (unreleased)     | `1.74.0`        |
| `0.13.0`         | `1.65.0`        |
| `0.12.1`         | `1.65.0`        |
| `0.12.0`         | `1.65.0`        |
| `0.11.0`         | `1.65.0`        |
| `0.10.0`         | `1.61.0`        |
| `0.9.0`          | `1.57.0`        |
| `0.8.0`          | `1.52.0`        |
| `0.7.0`          | `1.52.0`        |
| `0.6.0`          | `1.52.0`        |
| `0.5.0`          | `1.52.0`        |
| `0.4.0`          | `1.52.0`        |
| `0.3.0`          | `1.51.0`        |
| `0.2.0`          | `1.41.0`        |
| `0.1.0`          | `1.41.0`        |
| `0.0.4`          | `1.40.0`        |
| `0.0.3`          | `1.40.0`        |
| `0.0.2`          | `1.36.0`        |
| `0.0.1`          | `1.36.0`        |

## Policy

The table above will be kept up-to-date in lock-step with CI on the main branch in GitHub. It may contain information about unreleased and yanked versions. It is the user's responsibility to consult with the [`pixels` versions page](https://crates.io/crates/pixels/versions) on `crates.io` to verify version status.

The MSRV will be chosen as the minimum version of `rustc` that can successfully pass CI, including documentation, lints, and all examples. For this reason, the minimum version _supported_ may be higher than the minimum version _required_ to compile the `pixels` crate itself. See `Cargo.toml` for the minimal Rust version required to build the crate alone.
