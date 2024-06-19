//! Defines the R4L error type.
//!
//! Every OS should provides:
//! - An Error type 
//! - Errno: const variable in mod code 
//! 

#[cfg(feature = "starry")]
mod error{
    use axerrno::AxError;
    pub type Error = AxError;

    pub mod code {
        macro_rules! declare_err {
            ($err:ident, $doc:expr, $var_b:ident) => {
                #[doc = $doc]
                pub const $err: super::Error = super::Error::$var_b;
            };
        }

        declare_err!(EPERM, "Operation not permitted.",PermissionDenied);
        declare_err!(ENOENT, "No such file or directory.",NotFound);
        declare_err!(ESRCH, "No such process.", NotFound);
        declare_err!(EINTR, "Interrupted system call.", Interrupted);
        declare_err!(EIO, "I/O error.", Io);
        //declare_err!(ENXIO, "No such device or address.");
        //declare_err!(E2BIG, "Argument list too long.");
        //declare_err!(ENOEXEC, "Exec format error.");
        //declare_err!(EBADF, "Bad file number.");
        //declare_err!(ECHILD, "No child processes.");
        declare_err!(EAGAIN, "Try again.", Again);
        declare_err!(ENOMEM, "Out of memory.", NoMemory);
        declare_err!(EACCES, "Permission denied.", PermissionDenied);
        declare_err!(EFAULT, "Bad address.", BadState);
        //declare_err!(ENOTBLK, "Block device required.");
        declare_err!(EBUSY, "Device or resource busy.", Busy);
        declare_err!(EEXIST, "File exists.", AlreadyExists);
        //declare_err!(EXDEV, "Cross-device link.");
        //declare_err!(ENODEV, "No such device.");
        declare_err!(ENOTDIR, "Not a directory.", NotADirectory);
        //declare_err!(EISDIR, "Is a directory.");
        declare_err!(EINVAL, "Invalid argument.", InvalidInput);
        //declare_err!(ENFILE, "File table overflow.");
        //declare_err!(EMFILE, "Too many open files.");
        //declare_err!(ENOTTY, "Not a typewriter.");
        //declare_err!(ETXTBSY, "Text file busy.");
        //declare_err!(EFBIG, "File too large.");
        declare_err!(ENOSPC, "No space left on device.", StorageFull);
        //declare_err!(ESPIPE, "Illegal seek.");
        //declare_err!(EROFS, "Read-only file system.");
        //declare_err!(EMLINK, "Too many links.");
        //declare_err!(EPIPE, "Broken pipe.");
        //declare_err!(EDOM, "Math argument out of domain of func.");
        //declare_err!(ERANGE, "Math result not representable.");
        //declare_err!(ERESTARTSYS, "Restart the system call.");
        //declare_err!(ERESTARTNOINTR, "System call was interrupted by a signal and will be restarted.");
        //declare_err!(ERESTARTNOHAND, "Restart if no handler.");
        //declare_err!(ENOIOCTLCMD, "No ioctl command.");
        //declare_err!(ERESTART_RESTARTBLOCK, "Restart by calling sys_restart_syscall.");
        //declare_err!(EPROBE_DEFER, "Driver requests probe retry.");
        //declare_err!(EOPENSTALE, "Open found a stale dentry.");
        //declare_err!(ENOPARAM, "Parameter not supported.");
        //declare_err!(EBADHANDLE, "Illegal NFS file handle.");
        //declare_err!(ENOTSYNC, "Update synchronization mismatch.");
        //declare_err!(EBADCOOKIE, "Cookie is stale.");
        declare_err!(ENOTSUPP, "Operation is not supported.", Unsupported);
        // declare_err!(ETOOSMALL, "Buffer or request is too small.");
        //declare_err!(ESERVERFAULT, "An untranslatable error occurred.");
        // declare_err!(EBADTYPE, "Type not supported by server.");
        //declare_err!(EJUKEBOX, "Request initiated, but will not complete before timeout.");
        //declare_err!(EIOCBQUEUED, "iocb queued, will get completion event.");
        //declare_err!(ERECALLCONFLICT, "Conflict with recalled state.");
        //declare_err!(ENOGRACE, "NFS file lock reclaim refused.", );
    }

}

pub use error::Error;
pub use error::code;

/// A [`Result`] with an [`Error`] error type.
pub type Result<T> = core::result::Result<T, Error>;

