use crossbeam_channel::{bounded, select, Receiver, Sender};
use std::io::{self, Write};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct AsyncWriterConfig {
    pub buffer_capacity: usize,
    pub flush_interval_ms: u64,
    pub channel_capacity: usize,
}

impl Default for AsyncWriterConfig {
    fn default() -> Self {
        Self {
            buffer_capacity: 8192,
            flush_interval_ms: 100,
            channel_capacity: 10000,
        }
    }
}

enum WriterMessage {
    Data(Vec<u8>),
    Flush,
    Shutdown,
}

pub struct AsyncWriter {
    sender: Sender<WriterMessage>,
    _guard: Arc<WriterGuard>,
}

impl AsyncWriter {
    pub fn new() -> (Self, Arc<WriterGuard>) {
        Self::with_config(AsyncWriterConfig::default())
    }

    pub fn with_config(config: AsyncWriterConfig) -> (Self, Arc<WriterGuard>) {
        let (sender, receiver) = bounded(config.channel_capacity);

        let guard = Arc::new(WriterGuard {
            sender: sender.clone(),
        });

        let guard_clone = Arc::clone(&guard);

        let _handle = thread::Builder::new()
            .name("async-log-writer".to_string())
            .spawn(move || {
                background_writer(receiver, config);
            })
            .expect("Failed to spawn async writer thread");

        let writer = AsyncWriter {
            sender,
            _guard: guard_clone,
        };

        (writer, guard)
    }

    #[inline]
    pub fn write_data(&self, data: &[u8]) -> io::Result<()> {
        self.sender
            .try_send(WriterMessage::Data(data.to_vec()))
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "Writer thread closed"))
    }

    #[inline]
    pub fn flush_now(&self) -> io::Result<()> {
        self.sender
            .try_send(WriterMessage::Flush)
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "Writer thread closed"))
    }
}

impl Clone for AsyncWriter {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            _guard: Arc::clone(&self._guard),
        }
    }
}

impl Write for AsyncWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.write_data(buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.flush_now()
    }
}

pub struct WriterGuard {
    sender: Sender<WriterMessage>,
}

impl Drop for WriterGuard {
    fn drop(&mut self) {
        let _ = self.sender.send(WriterMessage::Shutdown);
    }
}

struct BufferedWriter {
    buffer: Vec<u8>,
    stdout: io::Stdout,
    capacity: usize,
}

impl BufferedWriter {
    #[inline]
    fn new(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
            stdout: io::stdout(),
            capacity,
        }
    }

    #[inline]
    fn push(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);
        if self.buffer.len() >= self.capacity {
            self.flush();
        }
    }

    #[inline]
    fn flush(&mut self) {
        if self.buffer.is_empty() {
            return;
        }
        if let Err(e) = self.stdout.write_all(&self.buffer) {
            eprintln!("Async writer error: {}", e);
        }
        let _ = self.stdout.flush();
        self.buffer.clear();
    }
}

fn background_writer(receiver: Receiver<WriterMessage>, config: AsyncWriterConfig) {
    let mut writer = BufferedWriter::new(config.buffer_capacity);
    let flush_interval = Duration::from_millis(config.flush_interval_ms);
    let ticker = crossbeam_channel::tick(flush_interval);

    loop {
        select! {
            recv(receiver) -> msg => {
                match msg {
                    Ok(m) => {
                        if handle_message(m, &mut writer) {
                            return;
                        }
                        while let Ok(m) = receiver.try_recv() {
                            if handle_message(m, &mut writer) {
                                return;
                            }
                        }
                    }
                    Err(_) => {
                        writer.flush();
                        return;
                    }
                }
            }
            recv(ticker) -> _ => {
                writer.flush();
            }
        }
    }
}

#[inline]
fn handle_message(msg: WriterMessage, writer: &mut BufferedWriter) -> bool {
    match msg {
        WriterMessage::Data(data) => {
            writer.push(&data);
            false
        }
        WriterMessage::Flush => {
            writer.flush();
            false
        }
        WriterMessage::Shutdown => {
            writer.flush();
            true
        }
    }
}

#[derive(Clone)]
pub struct AsyncMakeWriter {
    writer: AsyncWriter,
}

impl AsyncMakeWriter {
    pub fn new() -> (Self, Arc<WriterGuard>) {
        let (writer, guard) = AsyncWriter::new();
        (Self { writer }, guard)
    }

    pub fn with_config(config: AsyncWriterConfig) -> (Self, Arc<WriterGuard>) {
        let (writer, guard) = AsyncWriter::with_config(config);
        (Self { writer }, guard)
    }
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for AsyncMakeWriter {
    type Writer = AsyncWriter;

    fn make_writer(&'a self) -> Self::Writer {
        self.writer.clone()
    }
}
