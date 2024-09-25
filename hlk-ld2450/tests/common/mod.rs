pub struct MockSerial<'a, const LEN: usize> {
    data: &'a [u8],
    position: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MockSerialError;

impl embedded_io_async::Error for MockSerialError {
    fn kind(&self) -> embedded_io_async::ErrorKind {
        embedded_io_async::ErrorKind::Other
    }
}

impl<'a, const LEN: usize> MockSerial<'a, LEN> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, position: 0 }
    }
}

impl<const LEN: usize> embedded_io_async::Read for MockSerial<'_, LEN> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let view = &self.data[self.position..];
        let len = buf.len().min(view.len());
        buf[..len].copy_from_slice(&view[..len]);
        self.position += len;
        Ok(len)
    }
}

impl<const LEN: usize> embedded_io_async::ErrorType for MockSerial<'_, LEN> {
    type Error = MockSerialError;
}
