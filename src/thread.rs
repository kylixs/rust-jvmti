use super::native::JavaThread;

///
/// Represents a link between a JVM thread and the Rust code calling the JVMTI API.
///
#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct ThreadId {
    pub native_id: JavaThread,
}

/// Marker trait implementation for `Send`
unsafe impl Send for ThreadId { }

/// Marker trait implementation for `Sync`
unsafe impl Sync for ThreadId { }

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct Thread {
    pub id: ThreadId,
    pub name: String,
    pub priority: u32,
    pub is_daemon: bool
}
