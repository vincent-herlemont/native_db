set shell := ["nu", "-c"]

default:
    @just --list --unsorted;

build_no_default *args:
    cargo build --no-default-features {{args}}

# E.g. just build_default --test modules breaking_release_migration::from_0_5_x_to_0_6_x
build_default *args:
    cargo build {{args}}

build_with_optional *args:
    cargo build -F chrono -F uuid -F tokio {{args}}

build_all *args:
    just build_no_default {{args}};
    just build_default {{args}};
    just build_with_optional {{args}}

test_no_default *args:
    cargo test --no-default-features {{args}} -- --nocapture

test_default *args:
    cargo test {{args}} -- --nocapture

test_with_optional *args:
    cargo test -F chrono -F uuid -F tokio {{args}} -- --nocapture

test_all *args:
    just test_no_default {{args}};
    just test_default {{args}};
    just test_with_optional {{args}}


# List all available devices
test_mobile_all_devices:
    cargo dinghy all-devices

# List all available platforms
test_mobile_all_platforms:
    echo $env.ANDROID_NDK_HOME; \
    cargo dinghy all-platforms

[macos]
test_ios_launch_simulator device="iPhone 14":
    xcrun simctl boot "{{device}}"

[macos]
test_ios_list_simulators:
    xcrun simctl list

# args: E.g. "--test modules watch::watch_multithreading"
[macos]
test_ios *args:
    cargo dinghy -d iphone test {{args}}

# List all available android emulators
test_android_list_emulators:
    emulator -list-avds

# Launch android emulator
test_android_launch_emulator emulator="Pixel_3a_API_34_extension_level_7_arm64-v8a":
    emulator -avd "{{emulator}}"

# List all adb devices
test_android_list_devices:
    adb devices

test_android *args:
    cargo dinghy -d android test {{args}}

bench_build:
    cargo bench --no-run

bench:
    CRITERION_DEBUG=1 cargo bench; \
    start ./target/criterion/report/index.html

expand test_file_name:
    rm -f {{test_file_name}}.expanded.rs; \
    cargo expand --test {{test_file_name}} | save --raw {{test_file_name}}.expanded.rs