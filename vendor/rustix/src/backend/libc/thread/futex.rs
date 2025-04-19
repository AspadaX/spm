use crate::backend::c;

bitflags::bitflags! {
    /// `FUTEX_*` flags for use with the functions in [`futex`].
    ///
    /// [`futex`]: mod@crate::thread::futex
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
    pub struct Flags: u32 {
        /// `FUTEX_PRIVATE_FLAG`
        const PRIVATE = bitcast!(c::FUTEX_PRIVATE_FLAG);
        /// `FUTEX_CLOCK_REALTIME`
        const CLOCK_REALTIME = bitcast!(c::FUTEX_CLOCK_REALTIME);

        /// <https://docs.rs/bitflags/*/bitflags/#externally-defined-flags>
        const _ = !0;
    }
}

bitflags::bitflags! {
    /// `FUTEX2_*` flags for use with the functions in [`Waitv`].
    ///
    /// Not to be confused with [`WaitvFlags`], which is passed as an argument
    /// to the `waitv` function.
    ///
    /// [`Waitv`]: crate::thread::futex::Waitv
    /// [`WaitvFlags`]: crate::thread::futex::WaitvFlags
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
    pub struct WaitFlags: u32 {
        /// `FUTEX_U8`
        const SIZE_U8 = linux_raw_sys::general::FUTEX2_SIZE_U8;
        /// `FUTEX_U16`
        const SIZE_U16 = linux_raw_sys::general::FUTEX2_SIZE_U16;
        /// `FUTEX_U32`
        const SIZE_U32 = linux_raw_sys::general::FUTEX2_SIZE_U32;
        /// `FUTEX_U64`
        const SIZE_U64 = linux_raw_sys::general::FUTEX2_SIZE_U64;
        /// `FUTEX_SIZE_MASK`
        const SIZE_MASK = linux_raw_sys::general::FUTEX2_SIZE_MASK;

        /// `FUTEX2_NUMA`
        const NUMA = linux_raw_sys::general::FUTEX2_NUMA;

        /// `FUTEX2_PRIVATE`
        const PRIVATE = linux_raw_sys::general::FUTEX2_PRIVATE;

        /// <https://docs.rs/bitflags/*/bitflags/#externally-defined-flags>
        const _ = !0;
    }
}

/// `FUTEX_*` operations for use with the futex syscall wrappers.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u32)]
pub(crate) enum Operation {
    /// `FUTEX_WAIT`
    Wait = bitcast!(c::FUTEX_WAIT),
    /// `FUTEX_WAKE`
    Wake = bitcast!(c::FUTEX_WAKE),
    /// `FUTEX_FD`
    Fd = bitcast!(c::FUTEX_FD),
    /// `FUTEX_REQUEUE`
    Requeue = bitcast!(c::FUTEX_REQUEUE),
    /// `FUTEX_CMP_REQUEUE`
    CmpRequeue = bitcast!(c::FUTEX_CMP_REQUEUE),
    /// `FUTEX_WAKE_OP`
    WakeOp = bitcast!(c::FUTEX_WAKE_OP),
    /// `FUTEX_LOCK_PI`
    LockPi = bitcast!(c::FUTEX_LOCK_PI),
    /// `FUTEX_UNLOCK_PI`
    UnlockPi = bitcast!(c::FUTEX_UNLOCK_PI),
    /// `FUTEX_TRYLOCK_PI`
    TrylockPi = bitcast!(c::FUTEX_TRYLOCK_PI),
    /// `FUTEX_WAIT_BITSET`
    WaitBitset = bitcast!(c::FUTEX_WAIT_BITSET),
    /// `FUTEX_WAKE_BITSET`
    WakeBitset = bitcast!(c::FUTEX_WAKE_BITSET),
    /// `FUTEX_WAIT_REQUEUE_PI`
    WaitRequeuePi = bitcast!(c::FUTEX_WAIT_REQUEUE_PI),
    /// `FUTEX_CMP_REQUEUE_PI`
    CmpRequeuePi = bitcast!(c::FUTEX_CMP_REQUEUE_PI),
    /// `FUTEX_LOCK_PI2`
    LockPi2 = bitcast!(c::FUTEX_LOCK_PI2),
}

/// `FUTEX_WAITERS`
pub const WAITERS: u32 = linux_raw_sys::general::FUTEX_WAITERS;

/// `FUTEX_OWNER_DIED`
pub const OWNER_DIED: u32 = linux_raw_sys::general::FUTEX_OWNER_DIED;
