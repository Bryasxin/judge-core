//! Firecracker instance builder, which returns an unstarted firecracker wrapper
use crate::dto::LoggerLevel;
use std::path::PathBuf;

/// Used for quickly generating builder pattern setter methods
/// HACK: This is a temporary method and will be modified later.
macro_rules! with {
    // Match [`Option<T>`]
    ($field_name:expr, Option<$inner_type:ty>) => {
        paste::paste! {
            pub fn [<with_ $field_name>](&mut self, $field_name: $inner_type) -> &mut Self {
                self.[<$field_name>] = Some($field_name);
                self
            }
        }
    };
    ($method_name:expr, $field_name:expr, Option<$inner_type:ty>) => {
        paste::paste! {
            pub fn [<with_ $method_name>](&mut self, $field_name: $inner_type) -> &mut Self {
                self.[<$field_name>] = Some($field_name);
                self
            }
        }
    };

    // Match normal types
    ($field_name:expr, $field_type:ty) => {
        paste::paste! {
            pub fn [<with_ $field_name>](&mut self, $field_name: $field_type) -> &mut Self {
                self.[<$field_name>] = $field_name;
                self
            }
        }
    };
    ($method_name:expr, $field_name:expr, $field_type:ty) => {
        paste::paste! {
            pub fn [<with_ $method_name>](&mut self, $field_name: $field_type) -> &mut Self {
                self.[<$field_name>] = $field_name;
                self
            }
        }
    };
}

#[derive(Debug, Default, Clone)]
pub struct FirecrackerBuilder {
    firecracker_binary: PathBuf,

    /// Path to unix domain socket used by the api
    api_socket_path: Option<PathBuf>,
    /// Enable pci support
    enable_pci: bool,
    /// Path to a file that contains the microVM configuration in json format
    config_file: Option<PathBuf>,
    /// Loads boot timer device for logging elapsed time since `InstanceStart` command
    enable_boot_timer: bool,
    /// Print the data format version of the provided snapshot state file
    describe_snapshot_file: Option<PathBuf>,
    /// Instance id
    id: Option<String>,
    /// Logger level
    logger_level: Option<LoggerLevel>,
    /// Path to a fifo or a file used for configuring the logger on startup
    log_file: Option<PathBuf>,
    /// Path to a file that contains metadata in JSON format to add to the mmds
    metadata_file: Option<PathBuf>,
    /// Path to a fifo or a file used for configuring the metrics on startup
    metrics_file: Option<PathBuf>,
    /// Mmds data store limit, in bytes
    mmds_size_limit: Option<usize>,
    /// Http api request payload max size, in bytes
    http_api_max_payload_limit: Option<usize>,
    /// Logger module filter
    logger_module_filter: Option<String>,
    /// Disables seccomp
    disable_seccomp: Option<bool>,
    /// Parent process cpu time (wall clock, microseconds)
    parent_cpu_time: Option<usize>,
    /// Specifies the path to a custom seccomp filter
    seccomp_filter: Option<String>,
    /// Process start CPU time (wall clock, microseconds)
    start_time_cpu: Option<usize>,
    /// Process start time (wall clock, microseconds)
    start_time: Option<usize>,
    /// Outputs the level in the logs
    show_level: Option<bool>,
    /// Includes the file path and line number of the log's origin
    show_log_origin: Option<bool>,
}

impl FirecrackerBuilder {
    pub fn new(firecracker_binary: impl Into<PathBuf>) -> Self {
        Self {
            firecracker_binary: firecracker_binary.into(),
            ..Default::default()
        }
    }

    pub fn build(self) {
        todo!()
    }
}

impl FirecrackerBuilder {
    with!(firecracker_binary, PathBuf);
    with!(pci_support, enable_pci, bool);
    with!(api_socket_path, Option<PathBuf>);
    with!(config_file, Option<PathBuf>);
    with!(boot_timer, enable_boot_timer, bool);
    with!(describe_snapshot_file, Option<PathBuf>);
    with!(id, Option<String>);
    with!(logger_level, Option<LoggerLevel>);
    with!(log_file, Option<PathBuf>);
    with!(metadata_file, Option<PathBuf>);
    with!(metrics_file, Option<PathBuf>);
    with!(mmds_size_limit, Option<usize>);
    with!(http_api_max_payload_limit, Option<usize>);
    with!(logger_module_filter, Option<String>);
    with!(disable_seccomp, Option<bool>);
    with!(parent_cpu_time, Option<usize>);
    with!(seccomp_filter, Option<String>);
    with!(start_time_cpu, Option<usize>);
    with!(start_time, Option<usize>);
    with!(show_level, Option<bool>);
    with!(show_log_origin, Option<bool>);
}
