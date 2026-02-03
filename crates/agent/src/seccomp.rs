use libseccomp::{ScmpAction, ScmpFilterContext, ScmpSyscall, error::SeccompError};
use std::io;

fn seccomp_to_io_error(e: SeccompError) -> io::Error {
    io::Error::new(io::ErrorKind::Other, e)
}

#[derive(Debug)]
pub struct SeccompFilter;

impl SeccompFilter {
    /// Applies a basic seccomp filter that blocks dangerous syscalls.
    ///
    /// Block specific syscalls
    pub fn apply_basic_filter() -> io::Result<()> {
        let mut filter = ScmpFilterContext::new(ScmpAction::Allow).map_err(seccomp_to_io_error)?;

        // List of dangerous syscalls to block:
        // - File ops: open/creat/unlink/rmdir/mkdir - file creation/deletion
        // - Permission: chmod/chown/setuid/setgid - privilege changes
        // - System: mount/reboot/kexec - system-level operations
        // - Privilege: capset/ptrace - capability/ptrace debugging
        // - Network: socket/connect/bind/listen - network access
        let blocked_syscalls = [
            "open",
            "openat",
            "creat",
            "unlink",
            "unlinkat",
            "rmdir",
            "mkdir",
            "mkdirat",
            "chmod",
            "fchmod",
            "fchmodat",
            "chown",
            "fchown",
            "lchown",
            "fchownat",
            "setuid",
            "setgid",
            "setreuid",
            "setregid",
            "setgroups",
            "setresuid",
            "setresgid",
            "capset",
            "mount",
            "umount2",
            "pivot_root",
            "swapon",
            "swapoff",
            "reboot",
            "kexec_load",
            "kexec_file_load",
            "perf_event_open",
            "bpf",
            "ptrace",
            "process_vm_writev",
            "socket",
            "socketpair",
            "connect",
            "accept",
            "accept4",
            "bind",
            "listen",
        ];

        for syscall_name in blocked_syscalls {
            filter
                .add_rule(
                    ScmpAction::Errno(libc::EPERM),
                    ScmpSyscall::from_name(syscall_name).unwrap(),
                )
                .map_err(seccomp_to_io_error)?;
        }

        filter.load().map_err(seccomp_to_io_error)?;
        Ok(())
    }

    /// Applies a stricter whitelist-based filter.
    ///
    /// Only allowed specify syscalls
    pub fn apply_strict_filter() -> io::Result<()> {
        let mut filter =
            ScmpFilterContext::new(ScmpAction::Errno(libc::EPERM)).map_err(seccomp_to_io_error)?;

        // Whitelist: only these essential syscalls are allowed
        // - IO: read/write/close/pread64/pwrite64 - basic file operations
        // - Memory: brk/mmap/mprotect/munmap - memory management
        // - Process: exit/exit_group - process termination
        // - Signals: rt_sigaction/rt_sigprocmask/rt_sigreturn - signal handling
        // - Info: getpid/getuid/fstat - process info queries
        let allowed_syscalls = [
            "read",
            "write",
            "close",
            "exit",
            "exit_group",
            "brk",
            "mmap",
            "mprotect",
            "munmap",
            "fstat",
            "lseek",
            "getpid",
            "getppid",
            "getuid",
            "getgid",
            "geteuid",
            "getegid",
            "arch_prctl",
            "set_tid_address",
            "set_robust_list",
            "futex",
            "rt_sigaction",
            "rt_sigprocmask",
            "rt_sigreturn",
            "ioctl",
            "pread64",
            "pwrite64",
            "clock_gettime",
            "clock_nanosleep",
            "nanosleep",
            "gettimeofday",
            "time",
            "getrandom",
        ];

        for syscall_name in allowed_syscalls {
            filter
                .add_rule(
                    ScmpAction::Allow,
                    ScmpSyscall::from_name(syscall_name).unwrap(),
                )
                .map_err(seccomp_to_io_error)?;
        }

        filter.load().map_err(seccomp_to_io_error)?;
        Ok(())
    }
}
