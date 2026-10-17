//! Embedded terminal (PTY) support.

use std::io::{Read, Write};

use portable_pty::{CommandBuilder, PtySize, native_pty_system};

pub struct TerminalState {
    pub parser: vt100::Parser,
    pub writer: Box<dyn Write + Send>,
    pub rx: std::sync::mpsc::Receiver<Vec<u8>>,
    _child: Box<dyn portable_pty::Child + Send + Sync>,
    master: Box<dyn portable_pty::MasterPty + Send>,
}

impl TerminalState {
    pub fn new() -> Option<Self> {
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .ok()?;
        let cmd = CommandBuilder::new(
            std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string()),
        );
        let child = pair.slave.spawn_command(cmd).ok()?;
        let master = pair.master;
        let writer = master.take_writer().ok()?;
        let mut reader = master.try_clone_reader().ok()?;
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let _ = tx.send(buf[..n].to_vec());
                    }
                    Err(_) => break,
                }
            }
        });
        let parser = vt100::Parser::new(24, 80, 0);
        Some(Self {
            parser,
            writer,
            rx,
            _child: child,
            master,
        })
    }

    pub fn poll(&mut self) {
        while let Ok(bytes) = self.rx.try_recv() {
            self.parser.process(&bytes);
        }
    }

    pub fn write_input(&mut self, data: &[u8]) {
        let _ = self.writer.write_all(data);
        let _ = self.writer.flush();
    }

    pub fn resize(&mut self, rows: u16, cols: u16) {
        let _ = self.master.resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        });
        self.parser = vt100::Parser::new(rows, cols, 0);
    }
}