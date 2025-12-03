use actix::prelude::*;

#[derive(Message)]
#[rtype(result = "()")]
pub struct AcquireLock {
    pub(crate) from: u32,
    pub(crate) transaction_id: String,
    pub(crate) enterprise_id: u32,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ReleaseLock {
    pub(crate) transaction_id: String,
    pub(crate) enterprise_id: u32,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct NextLock {
    pub(crate) enterprise_id: u32,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct LockGranted {
    pub(crate) transaction_id: String,
    pub(crate) enterprise_id: u32,
}
