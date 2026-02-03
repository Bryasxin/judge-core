use libseccomp::{ScmpAction, ScmpFilterContext, ScmpSyscall, error::SeccompError};
use std::io;

fn seccomp_to_io_error(e: SeccompError) -> io::Error {
    io::Error::new(io::ErrorKind::Other, e)
}

#[derive(Debug)]
pub struct SeccompFilter;

impl SeccompFilter {
    pub fn apply_basic_filter() -> io::Result<()> {
        let mut filter = ScmpFilterContext::new(ScmpAction::Allow).map_err(seccomp_to_io_error)?;

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
            let syscall = ScmpSyscall::from_name(syscall_name).unwrap();

            filter
                .add_rule(ScmpAction::Errno(libc::EPERM), syscall)
                .map_err(seccomp_to_io_error)?;
        }

        filter.load().map_err(seccomp_to_io_error)?;
        Ok(())
    }

    pub fn apply_strict_filter() -> io::Result<()> {
        let mut filter = ScmpFilterContext::new(ScmpAction::Allow).map_err(seccomp_to_io_error)?;

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

        let syscall = ScmpSyscall::from_name("open").unwrap();
        filter
            .add_rule(ScmpAction::Errno(libc::EPERM), syscall)
            .map_err(seccomp_to_io_error)?;

        for syscall_name in allowed_syscalls {
            filter
                .add_rule(
                    ScmpAction::Allow,
                    ScmpSyscall::from_name(syscall_name).unwrap(),
                )
                .map_err(seccomp_to_io_error)?;
        }

        let blocked_syscalls = [
            "socket", "connect", "accept", "bind", "listen", "fork", "vfork", "clone", "execve",
            "execveat", "popen", "system", "setuid", "setgid", "setreuid", "setregid", "chmod",
            "chown", "open", "openat", "creat", "unlink", "rmdir", "mkdir", "mount", "umount2",
            "ptrace", "kill", "tkill", "tgkill",
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
}
