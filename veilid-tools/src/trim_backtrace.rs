use super::*;

pub struct TrimmedBacktrace {
    backtrace: backtrace::Backtrace,
}

impl TrimmedBacktrace {
    #[must_use]
    pub fn new(mut backtrace: backtrace::Backtrace) -> Self {
        backtrace.resolve();
        Self { backtrace }
    }

    #[must_use]
    pub fn backtrace(&self) -> &backtrace::Backtrace {
        &self.backtrace
    }

    #[must_use]
    pub fn backtrace_mut(&mut self) -> &mut backtrace::Backtrace {
        &mut self.backtrace
    }

    fn format_display(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let cwd = std::env::current_dir();
        let mut print_path =
            move |f: &mut fmt::Formatter<'_>, path: backtrace::BytesOrWideString<'_>| {
                let path = path.into_path_buf();
                if let Ok(cwd) = &cwd {
                    if let Ok(suffix) = path.strip_prefix(cwd) {
                        return fmt::Display::fmt(&suffix.display(), f);
                    }
                }
                fmt::Display::fmt(&path.display(), f)
            };

        let mut f = backtrace::BacktraceFmt::new(f, backtrace::PrintFmt::Short, &mut print_path);
        f.add_context()?;
        'frame_loop: for frame in self.backtrace.frames() {
            let mut frame_fmt = f.frame();

            let symbols = frame.symbols();
            for symbol in symbols {
                let symbol_name = symbol.name().map(|x| x.to_string()).unwrap_or_default();

                // Trim: Don't print lines with certain symbols
                if symbol_name.starts_with("backtrace::")
                    || symbol_name.starts_with("veilid_tools::async_locks::")
                    || symbol_name.starts_with("<tracing::instrument::Instrumented")
                    || symbol_name.starts_with("<core::pin::Pin<")
                    || symbol_name.starts_with("<core::panic::")
                    || symbol_name.starts_with("tokio::runtime::")
                    || symbol_name.starts_with("tokio::loom::")
                    || symbol_name.starts_with("std::panicking::")
                    || symbol_name.starts_with("std::panic::")
                {
                    continue 'frame_loop;
                }

                // Trim: Stop the backtrace at the first rust try/unwind frame
                if symbol_name.starts_with("___rust_try") {
                    break 'frame_loop;
                }

                frame_fmt.backtrace_symbol(frame, symbol)?;
            }
            if symbols.is_empty() {
                // Trim: don't print raw frames
                // frame_fmt.print_raw(frame.ip(), None, None, None)?;
            }
        }
        f.finish()?;
        Ok(())
    }
}

impl core::ops::Deref for TrimmedBacktrace {
    type Target = backtrace::Backtrace;

    fn deref(&self) -> &Self::Target {
        self.backtrace()
    }
}

impl core::ops::DerefMut for TrimmedBacktrace {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.backtrace_mut()
    }
}

pub trait TrimBacktraceTrait {
    fn trim(&self) -> TrimmedBacktrace;
}

impl TrimBacktraceTrait for backtrace::Backtrace {
    fn trim(&self) -> TrimmedBacktrace {
        TrimmedBacktrace::new(self.clone())
    }
}

impl From<TrimmedBacktrace> for backtrace::Backtrace {
    fn from(value: TrimmedBacktrace) -> Self {
        value.backtrace
    }
}

impl fmt::Display for TrimmedBacktrace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.format_display(f)
    }
}

impl fmt::Debug for TrimmedBacktrace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.backtrace.fmt(f)
    }
}
