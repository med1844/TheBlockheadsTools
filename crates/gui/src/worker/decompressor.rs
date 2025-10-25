use std::{collections::VecDeque, sync::mpsc};

use rayon::{ThreadPool, ThreadPoolBuilder};
use the_blockheads_tools_lib::{
    game::{chunk::Chunk, coord::ChunkCoord},
    util::gzip::FromGzip,
};

pub struct GzipDecompressWorker {
    thread_pool: ThreadPool,
    result_send: mpsc::Sender<(ChunkCoord, Chunk)>,
    result_recv: mpsc::Receiver<(ChunkCoord, Chunk)>,
    queue: VecDeque<ChunkCoord>,
    num_task_running: usize,
}

impl GzipDecompressWorker {
    pub fn new() -> Self {
        let (result_send, result_recv) = mpsc::channel();
        let thread_pool = ThreadPoolBuilder::new().build().unwrap();
        Self {
            thread_pool,
            result_send,
            result_recv,
            queue: VecDeque::new(),
            num_task_running: 0,
        }
    }

    pub fn add_coords<I: Iterator<Item = ChunkCoord>>(&mut self, i: I) {
        self.queue.extend(i);
    }

    pub fn need_byte_of_chunk(&mut self) -> Option<ChunkCoord> {
        (self.num_task_running < self.thread_pool.current_num_threads() << 1)
            .then(|| self.queue.pop_front())
            .flatten()
    }

    pub fn start_decompress(&mut self, coord: ChunkCoord, bytes: Vec<u8>) {
        let result_send = self.result_send.clone();
        self.thread_pool.install(move || {
            if let Ok(chunk) = Chunk::from_compressed_gzip(bytes.as_ref()) {
                let _ = result_send.send((coord, chunk));
            }
        });
        self.num_task_running += 1;
    }

    pub fn try_recv_chunk(&mut self) -> Option<(ChunkCoord, Chunk)> {
        self.result_recv.try_recv().ok().inspect(|_| {
            self.num_task_running -= 1;
        })
    }
}
