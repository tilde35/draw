use std::sync::Mutex;

pub(crate) struct PinnedData<T> {
    page_size: usize,
    data: Mutex<Vec<Vec<T>>>,
}
impl<T> PinnedData<T> {
    pub fn for_page_size(page_size: usize) -> Self {
        if page_size == 0 {
            panic!("Page size must be at least one");
        }
        Self {
            page_size,
            data: Mutex::new(Vec::new()),
        }
    }

    fn last_has_capacity(data: &Vec<Vec<T>>) -> bool {
        if let Some(last) = data.last() {
            last.len() < last.capacity()
        } else {
            false
        }
    }

    pub fn add<'a>(&'a self, v: T) -> &'a T {
        let mut data_loc = self.data.lock().unwrap();
        let data: &mut Vec<Vec<T>> = data_loc.as_mut();

        if !Self::last_has_capacity(data) {
            let new_row = Vec::with_capacity(self.page_size);
            data.push(new_row);
        }
        let row = data.len() - 1;
        let last = &mut data[row];
        let pos = last.len();
        last.push(v);
        // The data is pinned, so this is safe to elevate to 'a lifetime
        unsafe {
            let ptr: *const T = &last[pos];
            &(*ptr)
        }
    }
}
