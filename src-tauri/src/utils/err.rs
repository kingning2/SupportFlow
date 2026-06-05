use std::sync::Mutex;

pub fn lock_mutex<T>(m: &Mutex<T>) -> Result<std::sync::MutexGuard<'_, T>, String> {
    m.lock().map_err(|e| e.to_string())
}
