#[macro_export]
macro_rules! ready {
    ($ex:expr) => {
        match $ex {
            futures::task::Poll::Ready(r) => r,
            futures::task::Poll::Pending => return futures::task::Poll::Pending,
        }
    }
}

#[macro_export]
macro_rules! lock {
    ($ex:expr, $cx:expr) => {{
        let mut _lock = $ex.lock().boxed_local();
        ready!(_lock.poll_unpin($cx))
    }}
}

