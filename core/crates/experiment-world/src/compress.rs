//! Lossless evidence streaming using the already required Python standard library.
use std::{
    io::{self, BufWriter, Write},
    path::Path,
    process::{Child, ChildStdin, Command, Stdio},
};
pub struct Compressed {
    pipe: Option<BufWriter<ChildStdin>>,
    child: Child,
}
impl Compressed {
    pub fn create(path: &Path) -> io::Result<Self> {
        let mut child=Command::new("python").args(["-c","import gzip,shutil,sys; f=gzip.open(sys.argv[1],'xb',compresslevel=1); shutil.copyfileobj(sys.stdin.buffer,f); f.close()"]).arg(path).stdin(Stdio::piped()).spawn()?;
        let pipe =
            Some(BufWriter::new(child.stdin.take().ok_or_else(|| {
                io::Error::other("missing compression pipe")
            })?));
        Ok(Self { pipe, child })
    }
    pub fn finish(mut self) -> io::Result<()> {
        self.flush()?;
        drop(self.pipe.take());
        let status = self.child.wait()?;
        if status.success() {
            Ok(())
        } else {
            Err(io::Error::other("evidence compression failed"))
        }
    }
}
impl Write for Compressed {
    fn write(&mut self, b: &[u8]) -> io::Result<usize> {
        self.pipe
            .as_mut()
            .ok_or_else(|| io::Error::other("closed"))?
            .write(b)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.pipe
            .as_mut()
            .ok_or_else(|| io::Error::other("closed"))?
            .flush()
    }
}
