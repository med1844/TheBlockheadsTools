/// Simple arena for storing bytes contiguously.
pub(crate) struct ByteArena {
    buffer: Vec<u8>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct SliceId {
    start: usize,
    len: usize,
}

impl ByteArena {
    pub(crate) fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    pub(crate) fn add(&mut self, data: &[u8]) -> SliceId {
        let start = self.buffer.len();
        let len = data.len();
        self.buffer.extend_from_slice(data);
        SliceId { start, len }
    }

    pub(crate) fn get(&self, id: SliceId) -> &[u8] {
        &self.buffer[id.start..id.start + id.len]
    }
}
