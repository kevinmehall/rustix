#![allow(unsafe_code)]
use core::mem::{self, size_of};

use crate::backend::{
    c,
    net::write_sockaddr::{encode_sockaddr_v4, encode_sockaddr_v6},
};

use super::{SocketAddr, SocketAddrV4, SocketAddrV6};

#[cfg(unix)]
use super::SocketAddrUnix;

/// A trait abstracting over the types that can be passed as a `sockaddr`.
///
/// Safety: by implementing this trait, you assert that the values returned
/// by the trait methods can be passed to the system calls that accept `sockaddr`.
pub unsafe trait SocketAddress {
    /// The corresponding C `sockaddr_*` type.
    type CSockAddr;

    /// Convert to the C type.
    fn encode(&self) -> Self::CSockAddr;

    /// Call a closure with the pointer and length to the corresponding C type.
    /// This exists so types like `SockAddrUnix` that contain their corresponding
    /// C type can pass it directly without a copy.
    ///
    /// The default implementation passes a pointer to a stack variable containing the
    /// result of `encode`, and `size_of::<Self::CSockAddr>()`.
    fn with_sockaddr<R>(&self, f: impl FnOnce(*const c::sockaddr, c::socklen_t) -> R) -> R {
        let addr = self.encode();
        let ptr = (&addr as *const Self::CSockAddr).cast();
        let len = size_of::<Self::CSockAddr>() as c::socklen_t;
        f(ptr, len)
    }
}

unsafe impl SocketAddress for super::SocketAddr {
    type CSockAddr = c::sockaddr_storage;

    fn encode(&self) -> Self::CSockAddr {
        unsafe {
            let mut storage: c::sockaddr_storage = mem::zeroed();
            match self {
                SocketAddr::V4(v4) => {
                    let a = v4.encode();
                    core::ptr::write((&mut storage as *mut c::sockaddr_storage).cast(), a);
                }
                SocketAddr::V6(v6) => {
                    let a = v6.encode();
                    core::ptr::write((&mut storage as *mut c::sockaddr_storage).cast(), a);
                }
            }
            storage
        }
    }

    fn with_sockaddr<R>(&self, f: impl FnOnce(*const c::sockaddr, c::socklen_t) -> R) -> R {
        match self {
            SocketAddr::V4(v4) => v4.with_sockaddr(f),
            SocketAddr::V6(v6) => v6.with_sockaddr(f),
        }
    }
}

unsafe impl SocketAddress for SocketAddrV4 {
    type CSockAddr = c::sockaddr_in;

    fn encode(&self) -> Self::CSockAddr {
        encode_sockaddr_v4(self)
    }
}

unsafe impl SocketAddress for SocketAddrV6 {
    type CSockAddr = c::sockaddr_in6;

    fn encode(&self) -> Self::CSockAddr {
        encode_sockaddr_v6(self)
    }
}

#[cfg(unix)]
unsafe impl SocketAddress for SocketAddrUnix {
    type CSockAddr = c::sockaddr_un;

    fn encode(&self) -> Self::CSockAddr {
        self.unix
    }

    fn with_sockaddr<R>(&self, f: impl FnOnce(*const c::sockaddr, c::socklen_t) -> R) -> R {
        f(
            (&self.unix as *const c::sockaddr_un).cast(),
            self.addr_len(),
        )
    }
}
